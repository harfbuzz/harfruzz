use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use crate::hb::ot_layout_gsubgpos::{
    ligate_input, match_glyph, match_input, Apply, WouldApply, WouldApplyContext,
};
use read_fonts::tables::gsub::{Ligature, LigatureSet, LigatureSubstFormat1};
use read_fonts::types::GlyphId;

// TODO HarfBuzz caches covereage ala PairPos1

impl WouldApply for Ligature<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        let components = self.component_glyph_ids();
        ctx.glyphs.len() == components.len() + 1
            && components
                .iter()
                .map(|comp| GlyphId::from(comp.get()))
                .enumerate()
                .all(|(i, comp)| ctx.glyphs[i + 1] == comp)
    }
}

impl Apply for Ligature<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        // Special-case to make it in-place and not consider this
        // as a "ligated" substitution.
        let components = self.component_glyph_ids();
        if components.is_empty() {
            ctx.replace_glyph(self.ligature_glyph().into());
            Some(())
        } else {
            let f = |glyph, index| {
                let value = components.get(index as usize).unwrap().get().to_u16();
                match_glyph(glyph, value)
            };

            let mut match_end = 0;
            let mut match_positions = smallvec::SmallVec::from_elem(0, 4);
            let mut total_component_count = 0;

            if !match_input(
                ctx,
                components.len() as u16,
                &f,
                &mut match_end,
                &mut match_positions,
                Some(&mut total_component_count),
            ) {
                ctx.buffer
                    .unsafe_to_concat(Some(ctx.buffer.idx), Some(match_end));
                return None;
            }
            let count = components.len() + 1;
            ligate_input(
                ctx,
                count,
                &match_positions,
                match_end,
                total_component_count,
                self.ligature_glyph().into(),
            );
            Some(())
        }
    }
}

impl WouldApply for LigatureSet<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.ligatures()
            .iter()
            .filter_map(|lig| lig.ok())
            .any(|lig| lig.would_apply(ctx))
    }
}

impl Apply for LigatureSet<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        for lig in self.ligatures().iter().filter_map(|lig| lig.ok()) {
            if lig.apply(ctx).is_some() {
                return Some(());
            }
        }
        None
    }
}

impl WouldApply for LigatureSubstFormat1<'_> {
    fn would_apply(&self, ctx: &WouldApplyContext) -> bool {
        self.coverage()
            .ok()
            .and_then(|coverage| coverage.get(ctx.glyphs[0]))
            .and_then(|index| self.ligature_sets().get(index as usize).ok())
            .map_or(false, |set| set.would_apply(ctx))
    }
}

impl Apply for LigatureSubstFormat1<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        self.coverage()
            .ok()
            .and_then(|coverage| coverage.get(glyph))
            .and_then(|index| self.ligature_sets().get(index as usize).ok())
            .and_then(|set| set.apply(ctx))
    }
}
