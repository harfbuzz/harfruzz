use crate::hb::buffer::GlyphPropsFlags;
use crate::hb::ot_layout::{
    _hb_glyph_info_get_lig_id, _hb_glyph_info_is_ligature,
    _hb_glyph_info_set_lig_props_for_component,
};
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{Apply, WouldApply, WouldApplyContext};
use read_fonts::tables::gsub::MultipleSubstFormat1;

impl WouldApply for MultipleSubstFormat1<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        ctx.glyphs.len() == 1
            && self
                .coverage()
                .map(|cov| cov.get(ctx.glyphs[0]).is_some())
                .unwrap_or_default()
    }
}

impl Apply for MultipleSubstFormat1<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let gid = ctx.buffer.cur(0).as_glyph();
        let index = self.coverage().ok()?.get(gid)? as usize;
        let substs = self.sequences().get(index).ok()?.substitute_glyph_ids();
        match substs.len() {
            // Spec disallows this, but Uniscribe allows it.
            // https://github.com/harfbuzz/harfbuzz/issues/253
            0 => ctx.buffer.delete_glyph(),

            // Special-case to make it in-place and not consider this
            // as a "multiplied" substitution.
            1 => ctx.replace_glyph(substs.first()?.get().into()),

            _ => {
                let class = if _hb_glyph_info_is_ligature(ctx.buffer.cur(0)) {
                    GlyphPropsFlags::BASE_GLYPH
                } else {
                    GlyphPropsFlags::empty()
                };
                let lig_id = _hb_glyph_info_get_lig_id(ctx.buffer.cur(0));

                for (i, subst) in substs.iter().enumerate() {
                    let subst = subst.get().into();
                    // If is attached to a ligature, don't disturb that.
                    // https://github.com/harfbuzz/harfbuzz/issues/3069
                    if lig_id == 0 {
                        // Index is truncated to 4 bits anway, so we can safely cast to u8.
                        _hb_glyph_info_set_lig_props_for_component(ctx.buffer.cur_mut(0), i as u8);
                    }
                    ctx.output_glyph_for_component(subst, class);
                }

                ctx.buffer.skip_glyph();
            }
        }
        Some(())
    }
}
