use std::ops::Sub;

use crate::hb::{
    hb_font_t,
    ot_layout::LayoutLookup,
    ot_layout_gsubgpos::{Apply, WouldApply, WouldApplyContext, OT::hb_ot_apply_context_t},
    set_digest::hb_set_digest_ext,
};
use skrifa::raw::{
    tables::{
        gdef::Gdef,
        layout::{ClassDef, CoverageTable},
        variations::ItemVariationStore,
    },
    ReadError, TableProvider,
};
use ttf_parser::GlyphId;

mod contextual;
mod gpos;
mod gsub;
mod lookup_cache;

pub use gpos::GposTable;
pub use gsub::GsubTable;
pub use lookup_cache::{LookupCache, LookupInfo, Subtable};

#[derive(Clone)]
pub struct LayoutTables<'a> {
    pub gsub: Option<GsubTable<'a>>,
    pub gpos: Option<GposTable<'a>>,
    pub gdef: Option<Gdef<'a>>,
}

impl<'a> LayoutTables<'a> {
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            gsub: GsubTable::try_new(font),
            gpos: GposTable::try_new(font),
            gdef: font.gdef().ok(),
        }
    }

    pub fn item_variation_store(&self) -> Option<ItemVariationStore<'a>> {
        self.gdef
            .as_ref()
            .and_then(|gdef| gdef.item_var_store().transpose().ok().flatten())
    }
}

impl LayoutLookup for LookupInfo {
    fn props(&self) -> u32 {
        self.props
    }

    fn is_reverse(&self) -> bool {
        self.is_reversed
    }

    fn digest(&self) -> &crate::hb::set_digest::hb_set_digest_t {
        &self.digest
    }
}

impl Apply for LookupInfo {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        let glyph = ctx.buffer.cur(0).as_glyph();
        if !self.digest.may_have_glyph(glyph) {
            return None;
        }
        let (table_data, lookups) = if self.is_subst {
            let table = ctx.face.font.ot.gsub.as_ref()?;
            (table.table.offset_data().as_bytes(), &table.lookups)
        } else {
            let table = ctx.face.font.ot.gpos.as_ref()?;
            (table.table.offset_data().as_bytes(), &table.lookups)
        };
        let subtables = lookups.subtables(self)?;
        for subtable_info in subtables {
            if !subtable_info.digest.may_have_glyph(glyph) {
                continue;
            }
            // if subtable_info
            //     .primary_coverage(table_data, skrifa::GlyphId::from(glyph.0))
            //     .is_none()
            // {
            //     continue;
            // }
            let Ok(subtable) = subtable_info.materialize(table_data) else {
                continue;
            };
            let result = match subtable {
                Subtable::SingleSubst1(subtable) => subtable.apply(ctx),
                Subtable::SingleSubst2(subtable) => subtable.apply(ctx),
                Subtable::MultipleSubst1(subtable) => subtable.apply(ctx),
                Subtable::AlternateSubst1(subtable) => subtable.apply(ctx),
                Subtable::LigatureSubst1(subtable) => subtable.apply(ctx),
                Subtable::ReverseChainContext(subtable) => subtable.apply(ctx),
                Subtable::SinglePos1(subtable) => subtable.apply(ctx),
                Subtable::SinglePos2(subtable) => subtable.apply(ctx),
                Subtable::PairPos1(subtable) => subtable.apply(ctx),
                Subtable::PairPos2(subtable) => subtable.apply(ctx),
                Subtable::CursivePos1(subtable) => subtable.apply(ctx),
                Subtable::MarkBasePos1(subtable) => subtable.apply(ctx),
                Subtable::MarkLigPos1(subtable) => subtable.apply(ctx),
                Subtable::MarkMarkPos1(subtable) => subtable.apply(ctx),
                Subtable::ContextFormat1(subtable) => subtable.apply(ctx),
                Subtable::ContextFormat2(subtable) => subtable.apply(ctx),
                Subtable::ContextFormat3(subtable) => subtable.apply(ctx),
                Subtable::ChainedContextFormat1(subtable) => subtable.apply(ctx),
                Subtable::ChainedContextFormat2(subtable) => subtable.apply(ctx),
                Subtable::ChainedContextFormat3(subtable) => subtable.apply(ctx),
            };
            if result.is_some() {
                return Some(());
            }
        }
        None
    }
}

impl LookupInfo {
    pub fn would_apply(&self, face: &hb_font_t, ctx: &WouldApplyContext) -> Option<bool> {
        let glyph = ctx.glyphs[0];
        if !self.digest.may_have_glyph(glyph) {
            return Some(false);
        }
        let (table_data, lookups) = if self.is_subst {
            let table = face.font.ot.gsub.as_ref()?;
            (table.table.offset_data().as_bytes(), &table.lookups)
        } else {
            let table = face.font.ot.gpos.as_ref()?;
            (table.table.offset_data().as_bytes(), &table.lookups)
        };
        let subtables = lookups.subtables(self)?;
        for subtable_info in subtables {
            if !subtable_info.digest.may_have_glyph(glyph) {
                continue;
            }
            let Ok(subtable) = subtable_info.materialize(table_data) else {
                continue;
            };
            let result = match subtable {
                Subtable::SingleSubst1(subtable) => subtable.would_apply(ctx),
                Subtable::SingleSubst2(subtable) => subtable.would_apply(ctx),
                Subtable::MultipleSubst1(subtable) => subtable.would_apply(ctx),
                Subtable::AlternateSubst1(subtable) => subtable.would_apply(ctx),
                Subtable::LigatureSubst1(subtable) => subtable.would_apply(ctx),
                Subtable::ReverseChainContext(subtable) => subtable.would_apply(ctx),
                Subtable::ContextFormat1(subtable) => subtable.would_apply(ctx),
                Subtable::ContextFormat2(subtable) => subtable.would_apply(ctx),
                Subtable::ContextFormat3(subtable) => subtable.would_apply(ctx),
                Subtable::ChainedContextFormat1(subtable) => subtable.would_apply(ctx),
                Subtable::ChainedContextFormat2(subtable) => subtable.would_apply(ctx),
                Subtable::ChainedContextFormat3(subtable) => subtable.would_apply(ctx),
                _ => false,
            };
            return Some(result);
        }
        None
    }
}

fn coverage_index(coverage: Result<CoverageTable, ReadError>, gid: GlyphId) -> Option<u16> {
    let gid = skrifa::GlyphId16::new(gid.0);
    coverage.ok().and_then(|coverage| coverage.get(gid))
}

fn covered(coverage: Result<CoverageTable, ReadError>, gid: GlyphId) -> bool {
    coverage_index(coverage, gid).is_some()
}

fn glyph_class(class_def: Result<ClassDef, ReadError>, gid: GlyphId) -> u16 {
    let gid = skrifa::GlyphId16::new(gid.0);
    class_def
        .map(|class_def| class_def.get(gid))
        .unwrap_or_default()
}
