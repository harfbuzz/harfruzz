use crate::hb::ot_layout_gpos_table::ValueRecordExt;
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{skipping_iterator_t, Apply};
use skrifa::raw::tables::gpos::{PairPosFormat1, PairPosFormat2};

use super::Value;

impl Apply for PairPosFormat1<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let first_glyph = ctx.buffer.cur(0).as_skrifa_glyph();
        let first_glyph_coverage_index = self.coverage().ok()?.get(first_glyph)?;

        let mut iter = skipping_iterator_t::new(ctx, ctx.buffer.idx, false);

        let mut unsafe_to = 0;
        if !iter.next(Some(&mut unsafe_to)) {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(unsafe_to));
            return None;
        }

        let second_glyph_index = iter.index();
        let second_glyph = ctx.buffer.info[second_glyph_index].as_skrifa_glyph();

        let finish = |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, has_record2| {
            if has_record2 {
                *iter_index += 1;
                // https://github.com/harfbuzz/harfbuzz/issues/3824
                // https://github.com/harfbuzz/harfbuzz/issues/3888#issuecomment-1326781116
                ctx.buffer
                    .unsafe_to_break(Some(ctx.buffer.idx), Some(*iter_index + 1));
            }

            ctx.buffer.idx = *iter_index;

            Some(())
        };

        let boring = |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, has_record2| {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
            finish(ctx, iter_index, has_record2)
        };

        let success =
            |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, flag1, flag2, has_record2| {
                if flag1 || flag2 {
                    ctx.buffer
                        .unsafe_to_break(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
                    finish(ctx, iter_index, has_record2)
                } else {
                    boring(ctx, iter_index, has_record2)
                }
            };

        let bail =
            |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, records: (Value, Value)| {
                let flag1 = records.0.apply(ctx, ctx.buffer.idx);
                let flag2 = records.1.apply(ctx, second_glyph_index);

                let has_record2 = !records.1.is_empty();
                success(ctx, iter_index, flag1, flag2, has_record2)
            };

        let sets = self.pair_sets();
        let pair_sets = sets.get(first_glyph_coverage_index as usize).ok()?;
        let data = pair_sets.offset_data();
        let pair = pair_sets
            .pair_value_records()
            .iter()
            .filter_map(|value| value.ok())
            .find(|value| value.second_glyph() == second_glyph)?;
        let values = (
            Value {
                record: pair.value_record1,
                data,
            },
            Value {
                record: pair.value_record2,
                data,
            },
        );
        bail(ctx, &mut iter.buf_idx, values)
    }
}

impl Apply for PairPosFormat2<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let first_glyph = ctx.buffer.cur(0).as_skrifa_glyph();
        self.coverage().ok()?.get(first_glyph)?;

        let mut iter = skipping_iterator_t::new(ctx, ctx.buffer.idx, false);

        let mut unsafe_to = 0;
        if !iter.next(Some(&mut unsafe_to)) {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(unsafe_to));
            return None;
        }

        let second_glyph_index = iter.index();
        let second_glyph = ctx.buffer.info[second_glyph_index].as_skrifa_glyph();

        let finish = |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, has_record2| {
            if has_record2 {
                *iter_index += 1;
                // https://github.com/harfbuzz/harfbuzz/issues/3824
                // https://github.com/harfbuzz/harfbuzz/issues/3888#issuecomment-1326781116
                ctx.buffer
                    .unsafe_to_break(Some(ctx.buffer.idx), Some(*iter_index + 1));
            }

            ctx.buffer.idx = *iter_index;

            Some(())
        };

        let boring = |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, has_record2| {
            ctx.buffer
                .unsafe_to_concat(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
            finish(ctx, iter_index, has_record2)
        };

        let success =
            |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, flag1, flag2, has_record2| {
                if flag1 || flag2 {
                    ctx.buffer
                        .unsafe_to_break(Some(ctx.buffer.idx), Some(second_glyph_index + 1));
                    finish(ctx, iter_index, has_record2)
                } else {
                    boring(ctx, iter_index, has_record2)
                }
            };

        let bail =
            |ctx: &mut hb_ot_apply_context_t, iter_index: &mut usize, records: (Value, Value)| {
                let flag1 = records.0.apply(ctx, ctx.buffer.idx);
                let flag2 = records.1.apply(ctx, second_glyph_index);

                let has_record2 = !records.1.is_empty();
                success(ctx, iter_index, flag1, flag2, has_record2)
            };

        let class1 = self.class_def1().ok()?.get(first_glyph);
        let class2 = self.class_def2().ok()?.get(second_glyph);

        let data = self.offset_data();
        match self
            .class1_records()
            .get(class1 as usize)
            .and_then(|rec| rec.class2_records().get(class2 as usize))
            .ok()
        {
            Some(class2_record) => {
                let values = (
                    Value {
                        record: class2_record.value_record1,
                        data,
                    },
                    Value {
                        record: class2_record.value_record2,
                        data,
                    },
                );
                bail(ctx, &mut iter.buf_idx, values)
            }
            _ => {
                ctx.buffer
                    .unsafe_to_concat(Some(ctx.buffer.idx), Some(iter.index() + 1));
                return None;
            }
        }
    }
}
