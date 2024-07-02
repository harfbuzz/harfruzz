//! OpenType GPOS lookups.

use super::{LookupCache, LookupInfo};
use crate::hb::{
    ot_layout::TableIndex, ot_layout_gpos_table::ValueRecordExt,
    ot_layout_gsubgpos::OT::hb_ot_apply_context_t,
};
use skrifa::raw::{
    tables::{
        gpos::{DeviceOrVariationIndex, Gpos, ValueRecord},
        variations::DeltaSetIndex,
    },
    FontData, ReadError, TableProvider,
};

mod cursive;
mod mark;
mod pair;
mod single;

#[derive(Clone)]
pub struct GposTable<'a> {
    pub table: Gpos<'a>,
    pub lookups: LookupCache,
}

impl<'a> GposTable<'a> {
    pub fn try_new(font: &impl TableProvider<'a>) -> Option<Self> {
        let table = font.gpos().ok()?;
        let mut lookups = LookupCache::new();
        lookups.create_all(&table);
        Some(Self { table, lookups })
    }
}

impl<'a> crate::hb::ot_layout::LayoutTable for GposTable<'a> {
    const INDEX: TableIndex = TableIndex::GPOS;
    const IN_PLACE: bool = true;

    type Lookup = LookupInfo;

    fn get_lookup(&self, index: ttf_parser::opentype_layout::LookupIndex) -> Option<&Self::Lookup> {
        let lookup = self.lookups.get(index)?;
        if lookup.subtables_count == 0 {
            return None;
        }
        Some(lookup)
    }
}

struct Value<'a> {
    record: ValueRecord,
    data: FontData<'a>,
}

impl ValueRecordExt for Value<'_> {
    fn is_empty(&self) -> bool {
        self.record.x_placement.is_none()
            && self.record.y_placement.is_none()
            && self.record.x_advance.is_none()
            && self.record.y_advance.is_none()
            && self.record.x_placement_device.get().is_null()
            && self.record.y_placement_device.get().is_null()
            && self.record.x_advance_device.get().is_null()
            && self.record.y_advance_device.get().is_null()
    }

    fn apply(&self, ctx: &mut hb_ot_apply_context_t, idx: usize) -> bool {
        let mut pos = ctx.buffer.pos[idx];
        let worked = self.apply_to_pos(ctx, &mut pos);
        ctx.buffer.pos[idx] = pos;
        worked
    }

    fn apply_to_pos(
        &self,
        ctx: &mut hb_ot_apply_context_t,
        pos: &mut crate::GlyphPosition,
    ) -> bool {
        let horizontal = ctx.buffer.direction.is_horizontal();
        let mut worked = false;

        if let Some(value) = self.record.x_placement() {
            if value != 0 {
                pos.x_offset += i32::from(value);
                worked = true;
            }
        }

        if let Some(value) = self.record.y_placement() {
            if value != 0 {
                pos.y_offset += i32::from(value);
                worked = true;
            }
        }

        if horizontal {
            if let Some(value) = self.record.x_advance() {
                if value != 0 {
                    pos.x_advance += i32::from(value);
                    worked = true;
                }
            }
        } else {
            if let Some(value) = self.record.y_advance() {
                if value != 0 {
                    // y_advance values grow downward but font-space grows upward, hence negation
                    pos.y_advance -= i32::from(value);
                    worked = true;
                }
            }
        }

        if let Some(ivs) = ctx.face.font.ivs.as_ref() {
            let coords = &ctx.face.font.coords;
            let delta = |val: Result<DeviceOrVariationIndex<'_>, ReadError>| match val {
                Ok(DeviceOrVariationIndex::VariationIndex(varix)) => ivs
                    .compute_delta(
                        DeltaSetIndex {
                            outer: varix.delta_set_outer_index(),
                            inner: varix.delta_set_inner_index(),
                        },
                        coords,
                    )
                    .unwrap_or_default(),
                _ => 0,
            };

            let (ppem_x, ppem_y) = ctx.face.pixels_per_em().unwrap_or((0, 0));
            let coords = ctx.face.ttfp_face.variation_coordinates().len();
            let use_x_device = ppem_x != 0 || coords != 0;
            let use_y_device = ppem_y != 0 || coords != 0;

            if use_x_device {
                if let Some(device) = self.record.x_placement_device(self.data) {
                    pos.x_offset += delta(device);
                    worked = true; // TODO: even when 0?
                }
            }

            if use_y_device {
                if let Some(device) = self.record.y_placement_device(self.data) {
                    pos.y_offset += delta(device);
                    worked = true;
                }
            }

            if horizontal && use_x_device {
                if let Some(device) = self.record.x_advance_device(self.data) {
                    pos.x_advance += delta(device);
                    worked = true;
                }
            }

            if !horizontal && use_y_device {
                if let Some(device) = self.record.y_advance_device(self.data) {
                    // y_advance values grow downward but face-space grows upward, hence negation
                    pos.y_advance -= delta(device);
                    worked = true;
                }
            }
        }

        worked
    }
}
