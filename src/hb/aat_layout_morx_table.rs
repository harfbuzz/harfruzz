use super::aat_layout::*;
use super::aat_map::{hb_aat_map_builder_t, hb_aat_map_t, range_flags_t};
use super::{hb_font_t, hb_glyph_info_t};
use crate::hb::aat_layout_common::hb_aat_apply_context_t;
use crate::hb::ot_layout::MAX_CONTEXT_LENGTH;
use alloc::vec;
use read_fonts::tables::aat::{ExtendedStateTable, NoPayload, StateEntry};
use read_fonts::tables::morx::{
    ContextualEntryData, ContextualSubtable, InsertionEntryData, LigatureSubtable, SubtableKind,
};
use read_fonts::types::{BigEndian, FixedSize, GlyphId16};

// TODO: Use class cache, similarly to how it's used in harfbuzz.
// See https://github.com/harfbuzz/harfbuzz/pull/5031

// TODO: [morx] Blocklist dysfunctional morx table of AALMAGHRIBI.ttf font
// HarfBuzz commit 1e629c35113e2460fd4a77b4fa9ae3ff6ec876ba

// TODO: [morx] Only collect glyphs that can initiate action in the machine
// https://github.com/harfbuzz/harfbuzz/pull/5041

// Chain::compile_flags in harfbuzz
pub fn compile_flags(
    face: &hb_font_t,
    builder: &hb_aat_map_builder_t,
    map: &mut hb_aat_map_t,
) -> Option<()> {
    let has_feature = |kind: u16, setting: u16| {
        builder
            .current_features
            .binary_search_by(|probe| {
                if probe.kind != kind {
                    probe.kind.cmp(&kind)
                } else {
                    probe.setting.cmp(&setting)
                }
            })
            .is_ok()
    };

    let chains = face.aat_tables.morx.as_ref()?.chains();
    let chain_len = chains.iter().count();
    map.chain_flags.resize(chain_len, vec![]);

    for (chain, chain_flags) in chains.iter().zip(map.chain_flags.iter_mut()) {
        let Ok(chain) = chain else {
            continue;
        };
        let mut flags = chain.default_flags();
        for feature in chain.features() {
            // Check whether this type/setting pair was requested in the map,
            // and if so, apply its flags.

            if has_feature(feature.feature_type(), feature.feature_settings()) {
                flags &= feature.disable_flags();
                flags |= feature.enable_flags();
            } else if feature.feature_type() == HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE as u16
                && feature.feature_settings()
                    == u16::from(HB_AAT_LAYOUT_FEATURE_SELECTOR_SMALL_CAPS)
            {
                // Deprecated. https://github.com/harfbuzz/harfbuzz/issues/1342
                let ok = has_feature(
                    HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE as u16,
                    u16::from(HB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_SMALL_CAPS),
                );
                if ok {
                    flags &= feature.disable_flags();
                    flags |= feature.enable_flags();
                }
            }
            // TODO: Port the following commit: https://github.com/harfbuzz/harfbuzz/commit/2124ad890
        }

        chain_flags.push(range_flags_t {
            flags,
            cluster_first: builder.range_first as u32,
            cluster_last: builder.range_last as u32,
        });
    }

    Some(())
}

// Chain::apply in harfbuzz
pub fn apply<'a>(c: &mut hb_aat_apply_context_t<'a>, map: &'a mut hb_aat_map_t) -> Option<()> {
    c.buffer.unsafe_to_concat(None, None);

    let chains = c.face.aat_tables.morx.as_ref()?.chains();
    let chain_len = chains.iter().count();
    map.chain_flags.resize(chain_len, vec![]);

    for (chain, chain_flags) in chains.iter().zip(map.chain_flags.iter_mut()) {
        let Ok(chain) = chain else {
            continue;
        };
        c.range_flags = Some(chain_flags.as_mut_slice());
        for subtable in chain.subtables().iter() {
            let Ok(subtable) = subtable else {
                continue;
            };
            if let Some(range_flags) = c.range_flags.as_ref() {
                if range_flags.len() == 1
                    && (subtable.sub_feature_flags() & range_flags[0].flags == 0)
                {
                    continue;
                }
            }
            c.subtable_flags = subtable.sub_feature_flags();

            if !subtable.is_all_directions()
                && c.buffer.direction.is_vertical() != subtable.is_vertical()
            {
                continue;
            }

            // Buffer contents is always in logical direction.  Determine if
            // we need to reverse before applying this subtable.  We reverse
            // back after if we did reverse indeed.
            //
            // Quoting the spec:
            // """
            // Bits 28 and 30 of the coverage field control the order in which
            // glyphs are processed when the subtable is run by the layout engine.
            // Bit 28 is used to indicate if the glyph processing direction is
            // the same as logical order or layout order. Bit 30 is used to
            // indicate whether glyphs are processed forwards or backwards within
            // that order.
            //
            // Bit 30   Bit 28   Interpretation for Horizontal Text
            //      0        0   The subtable is processed in layout order
            //                   (the same order as the glyphs, which is
            //                   always left-to-right).
            //      1        0   The subtable is processed in reverse layout order
            //                   (the order opposite that of the glyphs, which is
            //                   always right-to-left).
            //      0        1   The subtable is processed in logical order
            //                   (the same order as the characters, which may be
            //                   left-to-right or right-to-left).
            //      1        1   The subtable is processed in reverse logical order
            //                   (the order opposite that of the characters, which
            //                   may be right-to-left or left-to-right).

            let reverse = if subtable.is_logical() {
                subtable.is_backwards()
            } else {
                subtable.is_backwards() != c.buffer.direction.is_backward()
            };

            if reverse {
                c.buffer.reverse();
            }

            if let Ok(kind) = subtable.kind() {
                apply_subtable(kind, c);
            }

            if reverse {
                c.buffer.reverse();
            }
        }
    }

    Some(())
}

trait driver_context_t<T> {
    fn in_place(&self) -> bool;
    fn can_advance(&self, entry: &StateEntry<T>) -> bool;
    fn is_actionable(&self, entry: &StateEntry<T>) -> bool;
    fn transition(&mut self, entry: &StateEntry<T>, ac: &mut hb_aat_apply_context_t) -> Option<()>;
}

const START_OF_TEXT: u16 = 0;

fn drive<T: bytemuck::AnyBitPattern + FixedSize + core::fmt::Debug>(
    machine: &ExtendedStateTable<'_, T>,
    c: &mut dyn driver_context_t<T>,
    ac: &mut hb_aat_apply_context_t,
) {
    if !c.in_place() {
        ac.buffer.clear_output();
    }

    let mut state = START_OF_TEXT;
    let mut last_range = ac.range_flags.as_ref().and_then(|rf| {
        if rf.len() > 1 {
            rf.first().map(|_| 0usize)
        } else {
            // If there's only one range, we already checked the flag.
            None
        }
    });
    ac.buffer.idx = 0;
    loop {
        // This block copied from NoncontextualSubtable::apply. Keep in sync.
        if let Some(range_flags) = ac.range_flags.as_ref() {
            if let Some(last_range) = last_range.as_mut() {
                let mut range = *last_range;
                if ac.buffer.idx < ac.buffer.len {
                    let cluster = ac.buffer.cur(0).cluster;
                    while cluster < range_flags[range].cluster_first {
                        range -= 1;
                    }

                    while cluster > range_flags[range].cluster_last {
                        range += 1;
                    }

                    *last_range = range;
                }

                if range_flags[range].flags & ac.subtable_flags == 0 {
                    if ac.buffer.idx == ac.buffer.len || !ac.buffer.successful {
                        break;
                    }

                    state = START_OF_TEXT;

                    ac.buffer.next_glyph();
                    continue;
                }
            }
        }

        let class = if ac.buffer.idx < ac.buffer.len {
            machine.class(ac.buffer.cur(0).as_glyph()).unwrap_or(1)
        } else {
            u16::from(read_fonts::tables::aat::class::END_OF_TEXT)
        };

        let entry = match machine.entry(state, class) {
            Ok(v) => v,
            _ => break,
        };

        let next_state = entry.new_state;

        // Conditions under which it's guaranteed safe-to-break before current glyph:
        //
        // 1. There was no action in this transition; and
        //
        // 2. If we break before current glyph, the results will be the same. That
        //    is guaranteed if:
        //
        //    2a. We were already in start-of-text state; or
        //
        //    2b. We are epsilon-transitioning to start-of-text state; or
        //
        //    2c. Starting from start-of-text state seeing current glyph:
        //
        //        2c'. There won't be any actions; and
        //
        //        2c". We would end up in the same state that we were going to end up
        //             in now, including whether epsilon-transitioning.
        //
        //    and
        //
        // 3. If we break before current glyph, there won't be any end-of-text action
        //    after previous glyph.
        //
        // This triples the transitions we need to look up, but is worth returning
        // granular unsafe-to-break results. See eg.:
        //
        //   https://github.com/harfbuzz/harfbuzz/issues/2860

        // TODO HarfBuzz doesn't use this lambda; inlines the logic.
        let is_safe_to_break_extra = || {
            // 2c
            let wouldbe_entry = match machine.entry(START_OF_TEXT, class) {
                Ok(v) => v,
                _ => return false,
            };

            // 2c'
            if c.is_actionable(&wouldbe_entry) {
                return false;
            }

            // 2c"
            next_state == wouldbe_entry.new_state
                && c.can_advance(&entry) == c.can_advance(&wouldbe_entry)
        };

        let is_safe_to_break =
            // 1
            !c.is_actionable(&entry) &&

            // 2
            (
                state == START_OF_TEXT
                || (!c.can_advance(&entry) && next_state == START_OF_TEXT)
                || is_safe_to_break_extra()
            ) &&

            // 3
            (
                if let Ok(end_entry) = machine.entry(state, u16::from(read_fonts::tables::aat::class::END_OF_TEXT)) {
                    !c.is_actionable(&end_entry)
                } else {
                    false
                }
            )
        ;

        if !is_safe_to_break && ac.buffer.backtrack_len() > 0 && ac.buffer.idx < ac.buffer.len {
            ac.buffer.unsafe_to_break_from_outbuffer(
                Some(ac.buffer.backtrack_len() - 1),
                Some(ac.buffer.idx + 1),
            );
        }

        c.transition(&entry, ac);

        state = next_state;

        if ac.buffer.idx >= ac.buffer.len || !ac.buffer.successful {
            break;
        }

        if c.can_advance(&entry) {
            ac.buffer.next_glyph();
        } else {
            if ac.buffer.max_ops <= 0 {
                ac.buffer.next_glyph();
            }
            ac.buffer.max_ops -= 1;
        }
    }

    if !c.in_place() {
        ac.buffer.sync();
    }
}

fn apply_subtable<'a>(kind: SubtableKind<'a>, ac: &mut hb_aat_apply_context_t<'a>) {
    match kind {
        SubtableKind::Rearrangement(table) => {
            let mut c = RearrangementCtx { start: 0, end: 0 };
            drive(&table, &mut c, ac);
        }
        SubtableKind::Contextual(table) => {
            let mut c = ContextualCtx {
                mark_set: false,
                mark: 0,
                table: table.clone(),
            };
            drive(&table.state_table, &mut c, ac);
        }
        SubtableKind::Ligature(table) => {
            let mut c = LigatureCtx {
                table: table.clone(),
                match_length: 0,
                match_positions: [0; LIGATURE_MAX_MATCHES],
            };
            drive(&table.state_table, &mut c, ac);
        }
        SubtableKind::NonContextual(ref lookup) => {
            let mut last_range = ac.range_flags.as_ref().and_then(|rf| {
                if rf.len() > 1 {
                    rf.first().map(|_| 0usize)
                } else {
                    // If there's only one range, we already checked the flag.
                    None
                }
            });

            for i in 0..ac.buffer.len {
                // This block copied from StateTableDriver::drive. Keep in sync.
                if let Some(range_flags) = ac.range_flags.as_ref() {
                    if let Some(last_range) = last_range.as_mut() {
                        let mut range = *last_range;
                        if ac.buffer.idx < ac.buffer.len {
                            // We need to access info
                            let cluster = ac.buffer.cur(0).cluster;
                            while cluster < range_flags[range].cluster_first {
                                range -= 1;
                            }

                            while cluster > range_flags[range].cluster_last {
                                range += 1;
                            }

                            *last_range = range;
                        }

                        if range_flags[range].flags & ac.subtable_flags == 0 {
                            continue;
                        }
                    }
                }

                if let Some(glyph) = ac.buffer.info[i].as_gid16() {
                    if let Ok(replacement) = lookup.value(glyph.to_u16()) {
                        ac.replace_glyph_inplace(i, replacement.into());
                    }
                }
            }
        }
        SubtableKind::Insertion(table) => {
            let mut c = InsertionCtx {
                mark: 0,
                glyphs: table.glyphs,
            };
            drive(&table.state_table, &mut c, ac);
        }
    }
}

struct RearrangementCtx {
    start: usize,
    end: usize,
}

impl RearrangementCtx {
    const MARK_FIRST: u16 = 0x8000;
    const DONT_ADVANCE: u16 = 0x4000;
    const MARK_LAST: u16 = 0x2000;
    const VERB: u16 = 0x000F;
}

impl driver_context_t<NoPayload> for RearrangementCtx {
    fn in_place(&self) -> bool {
        true
    }

    fn can_advance(&self, entry: &StateEntry) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &StateEntry) -> bool {
        entry.flags & Self::VERB != 0 && self.start < self.end
    }

    fn transition(&mut self, entry: &StateEntry, ac: &mut hb_aat_apply_context_t) -> Option<()> {
        let buffer = &mut ac.buffer;
        let flags = entry.flags;

        if flags & Self::MARK_FIRST != 0 {
            self.start = buffer.idx;
        }

        if flags & Self::MARK_LAST != 0 {
            self.end = (buffer.idx + 1).min(buffer.len);
        }

        if flags & Self::VERB != 0 && self.start < self.end {
            // The following map has two nibbles, for start-side
            // and end-side. Values of 0,1,2 mean move that many
            // to the other side. Value of 3 means move 2 and
            // flip them.
            const MAP: [u8; 16] = [
                0x00, // 0  no change
                0x10, // 1  Ax => xA
                0x01, // 2  xD => Dx
                0x11, // 3  AxD => DxA
                0x20, // 4  ABx => xAB
                0x30, // 5  ABx => xBA
                0x02, // 6  xCD => CDx
                0x03, // 7  xCD => DCx
                0x12, // 8  AxCD => CDxA
                0x13, // 9  AxCD => DCxA
                0x21, // 10 ABxD => DxAB
                0x31, // 11 ABxD => DxBA
                0x22, // 12 ABxCD => CDxAB
                0x32, // 13 ABxCD => CDxBA
                0x23, // 14 ABxCD => DCxAB
                0x33, // 15 ABxCD => DCxBA
            ];

            let m = MAP[usize::from(flags & Self::VERB)];
            let l = 2.min(m >> 4) as usize;
            let r = 2.min(m & 0x0F) as usize;
            let reverse_l = 3 == (m >> 4);
            let reverse_r = 3 == (m & 0x0F);

            if (self.end - self.start >= l + r) && (self.end - self.start <= MAX_CONTEXT_LENGTH) {
                buffer.merge_clusters(self.start, (buffer.idx + 1).min(buffer.len));
                buffer.merge_clusters(self.start, self.end);

                let mut buf = [hb_glyph_info_t::default(); 4];

                for (i, glyph_info) in buf[..l].iter_mut().enumerate() {
                    *glyph_info = buffer.info[self.start + i];
                }

                for i in 0..r {
                    buf[i + 2] = buffer.info[self.end - r + i];
                }

                if l > r {
                    for i in 0..(self.end - self.start - l - r) {
                        buffer.info[self.start + r + i] = buffer.info[self.start + l + i];
                    }
                } else if l < r {
                    for i in (0..(self.end - self.start - l - r)).rev() {
                        buffer.info[self.start + r + i] = buffer.info[self.start + l + i];
                    }
                }

                for i in 0..r {
                    buffer.info[self.start + i] = buf[2 + i];
                }

                for i in 0..l {
                    buffer.info[self.end - l + i] = buf[i];
                }

                if reverse_l {
                    buffer.info.swap(self.end - 1, self.end - 2);
                }

                if reverse_r {
                    buffer.info.swap(self.start, self.start + 1);
                }
            }
        }

        Some(())
    }
}

struct ContextualCtx<'a> {
    mark_set: bool,
    mark: usize,
    table: ContextualSubtable<'a>,
}

impl ContextualCtx<'_> {
    const SET_MARK: u16 = 0x8000;
    const DONT_ADVANCE: u16 = 0x4000;
}

impl driver_context_t<ContextualEntryData> for ContextualCtx<'_> {
    fn in_place(&self) -> bool {
        true
    }

    fn can_advance(&self, entry: &StateEntry<ContextualEntryData>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &StateEntry<ContextualEntryData>) -> bool {
        entry.payload.mark_index.get() != 0xFFFF || entry.payload.current_index.get() != 0xFFFF
    }

    fn transition(
        &mut self,
        entry: &StateEntry<ContextualEntryData>,
        ac: &mut hb_aat_apply_context_t,
    ) -> Option<()> {
        // Looks like CoreText applies neither mark nor current substitution for
        // end-of-text if mark was not explicitly set.
        if ac.buffer.idx == ac.buffer.len && !self.mark_set {
            return Some(());
        }

        let mut replacement = None;

        if entry.payload.mark_index.get() != 0xFFFF {
            let lookup = self
                .table
                .lookups
                .get(usize::from(entry.payload.mark_index.get()))
                .ok()?;
            if let Some(gid) = ac.buffer.info[self.mark].as_gid16() {
                replacement = lookup.value(gid.to_u16()).ok();
            }
        }

        if let Some(replacement) = replacement {
            ac.buffer.unsafe_to_break(
                Some(self.mark),
                Some((ac.buffer.idx + 1).min(ac.buffer.len)),
            );
            ac.replace_glyph_inplace(self.mark, replacement.into());
        }

        replacement = None;
        let idx = ac.buffer.idx.min(ac.buffer.len - 1);
        if entry.payload.current_index.get() != 0xFFFF {
            let lookup = self
                .table
                .lookups
                .get(usize::from(entry.payload.current_index.get()))
                .ok()?;
            if let Some(gid) = ac.buffer.info[idx].as_gid16() {
                replacement = lookup.value(gid.to_u16()).ok();
            }
        }

        if let Some(replacement) = replacement {
            ac.replace_glyph_inplace(idx, replacement.into());
        }

        if entry.flags & Self::SET_MARK != 0 {
            self.mark_set = true;
            self.mark = ac.buffer.idx;
        }

        Some(())
    }
}

struct InsertionCtx<'a> {
    mark: u32,
    glyphs: &'a [BigEndian<GlyphId16>],
}

impl InsertionCtx<'_> {
    const SET_MARK: u16 = 0x8000;
    const DONT_ADVANCE: u16 = 0x4000;
    const CURRENT_INSERT_BEFORE: u16 = 0x0800;
    const MARKED_INSERT_BEFORE: u16 = 0x0400;
    const CURRENT_INSERT_COUNT: u16 = 0x03E0;
    const MARKED_INSERT_COUNT: u16 = 0x001F;
}

impl driver_context_t<InsertionEntryData> for InsertionCtx<'_> {
    fn in_place(&self) -> bool {
        false
    }

    fn can_advance(&self, entry: &StateEntry<InsertionEntryData>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &StateEntry<InsertionEntryData>) -> bool {
        (entry.flags & (Self::CURRENT_INSERT_COUNT | Self::MARKED_INSERT_COUNT) != 0)
            && (entry.payload.current_insert_index.get() != 0xFFFF
                || entry.payload.marked_insert_index.get() != 0xFFFF)
    }

    fn transition(
        &mut self,
        entry: &StateEntry<InsertionEntryData>,
        ac: &mut hb_aat_apply_context_t,
    ) -> Option<()> {
        let flags = entry.flags;
        let mark_loc = ac.buffer.out_len;

        if entry.payload.marked_insert_index.get() != 0xFFFF {
            let count = flags & Self::MARKED_INSERT_COUNT;
            ac.buffer.max_ops -= i32::from(count);
            if ac.buffer.max_ops <= 0 {
                return Some(());
            }

            let start = entry.payload.marked_insert_index.get();
            let before = flags & Self::MARKED_INSERT_BEFORE != 0;

            let end = ac.buffer.out_len;
            ac.buffer.move_to(self.mark as usize);

            if ac.buffer.idx < ac.buffer.len && !before {
                ac.buffer.copy_glyph();
            }

            // TODO We ignore KashidaLike setting.
            for i in 0..count {
                let i = usize::from(start + i);
                ac.output_glyph(u32::from(self.glyphs.get(i)?.get().to_u16()));
            }

            if ac.buffer.idx < ac.buffer.len && !before {
                ac.buffer.skip_glyph();
            }

            ac.buffer.move_to(end + usize::from(count));

            ac.buffer.unsafe_to_break_from_outbuffer(
                Some(self.mark as usize),
                Some((ac.buffer.idx + 1).min(ac.buffer.len)),
            );
        }

        if flags & Self::SET_MARK != 0 {
            self.mark = mark_loc as u32;
        }

        if entry.payload.current_insert_index.get() != 0xFFFF {
            let count = (flags & Self::CURRENT_INSERT_COUNT) >> 5;
            ac.buffer.max_ops -= i32::from(count);
            if ac.buffer.max_ops < 0 {
                return Some(());
            }

            let start = entry.payload.current_insert_index.get();
            let before = flags & Self::CURRENT_INSERT_BEFORE != 0;
            let end = ac.buffer.out_len;

            if ac.buffer.idx < ac.buffer.len && !before {
                ac.buffer.copy_glyph();
            }

            // TODO We ignore KashidaLike setting.
            for i in 0..count {
                let i = usize::from(start + i);
                ac.output_glyph(u32::from(self.glyphs.get(i)?.get().to_u16()));
            }

            if ac.buffer.idx < ac.buffer.len && !before {
                ac.buffer.skip_glyph();
            }

            // Humm. Not sure where to move to. There's this wording under
            // DontAdvance flag:
            //
            // "If set, don't update the glyph index before going to the new state.
            // This does not mean that the glyph pointed to is the same one as
            // before. If you've made insertions immediately downstream of the
            // current glyph, the next glyph processed would in fact be the first
            // one inserted."
            //
            // This suggests that if DontAdvance is NOT set, we should move to
            // end+count. If it *was*, then move to end, such that newly inserted
            // glyphs are now visible.
            //
            // https://github.com/harfbuzz/harfbuzz/issues/1224#issuecomment-427691417
            ac.buffer.move_to(if flags & Self::DONT_ADVANCE != 0 {
                end
            } else {
                end + usize::from(count)
            });
        }

        Some(())
    }
}

const LIGATURE_MAX_MATCHES: usize = 64;

struct LigatureCtx<'a> {
    table: LigatureSubtable<'a>,
    match_length: usize,
    match_positions: [usize; LIGATURE_MAX_MATCHES],
}

impl LigatureCtx<'_> {
    const SET_COMPONENT: u16 = 0x8000;
    const DONT_ADVANCE: u16 = 0x4000;
    const PERFORM_ACTION: u16 = 0x2000;

    const LIG_ACTION_LAST: u32 = 0x80000000;
    const LIG_ACTION_STORE: u32 = 0x40000000;
    const LIG_ACTION_OFFSET: u32 = 0x3FFFFFFF;
}

impl driver_context_t<BigEndian<u16>> for LigatureCtx<'_> {
    fn in_place(&self) -> bool {
        false
    }

    fn can_advance(&self, entry: &StateEntry<BigEndian<u16>>) -> bool {
        entry.flags & Self::DONT_ADVANCE == 0
    }

    fn is_actionable(&self, entry: &StateEntry<BigEndian<u16>>) -> bool {
        entry.flags & Self::PERFORM_ACTION != 0
    }

    fn transition(
        &mut self,
        entry: &StateEntry<BigEndian<u16>>,
        ac: &mut hb_aat_apply_context_t,
    ) -> Option<()> {
        if entry.flags & Self::SET_COMPONENT != 0 {
            // Never mark same index twice, in case DONT_ADVANCE was used...
            if self.match_length != 0
                && self.match_positions[(self.match_length - 1) % LIGATURE_MAX_MATCHES]
                    == ac.buffer.out_len
            {
                self.match_length -= 1;
            }

            self.match_positions[self.match_length % LIGATURE_MAX_MATCHES] = ac.buffer.out_len;
            self.match_length += 1;
        }

        if entry.flags & Self::PERFORM_ACTION != 0 {
            let end = ac.buffer.out_len;

            if self.match_length == 0 {
                return Some(());
            }

            if ac.buffer.idx >= ac.buffer.len {
                return Some(()); // TODO: Work on previous instead?
            }

            let mut cursor = self.match_length;

            let mut ligature_actions_index = entry.payload.get();
            let mut ligature_idx = 0;
            loop {
                if cursor == 0 {
                    // Stack underflow. Clear the stack.
                    self.match_length = 0;
                    break;
                }

                cursor -= 1;
                ac.buffer
                    .move_to(self.match_positions[cursor % LIGATURE_MAX_MATCHES]);

                // We cannot use ? in this loop, because we must call
                // ac.buffer.move_to(end) in the end.
                let action = match self
                    .table
                    .ligature_actions
                    .get(usize::from(ligature_actions_index))
                {
                    Some(v) => v.get(),
                    None => break,
                };

                let mut uoffset = action & Self::LIG_ACTION_OFFSET;
                if uoffset & 0x20000000 != 0 {
                    uoffset |= 0xC0000000; // Sign-extend.
                }

                let offset = uoffset as i32;
                let component_idx = (ac.buffer.cur(0).glyph_id as i32 + offset) as usize;
                ligature_idx += match self.table.components.get(component_idx) {
                    Some(v) => v.get(),
                    None => break,
                };

                if (action & (Self::LIG_ACTION_STORE | Self::LIG_ACTION_LAST)) != 0 {
                    let lig = match self.table.ligatures.get(usize::from(ligature_idx)) {
                        Some(v) => v.get(),
                        None => break,
                    };

                    ac.replace_glyph(u32::from(lig.to_u16()));

                    let lig_end =
                        self.match_positions[(self.match_length - 1) % LIGATURE_MAX_MATCHES] + 1;
                    // Now go and delete all subsequent components.
                    while self.match_length - 1 > cursor {
                        self.match_length -= 1;
                        ac.buffer.move_to(
                            self.match_positions[self.match_length % LIGATURE_MAX_MATCHES],
                        );
                        ac.delete_glyph();
                    }

                    ac.buffer.move_to(lig_end);
                    ac.buffer.merge_out_clusters(
                        self.match_positions[cursor % LIGATURE_MAX_MATCHES],
                        ac.buffer.out_len,
                    );
                }

                ligature_actions_index += 1;

                if action & Self::LIG_ACTION_LAST != 0 {
                    break;
                }
            }

            ac.buffer.move_to(end);
        }

        Some(())
    }
}
