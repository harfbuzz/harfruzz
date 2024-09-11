use crate::hb::{
    common::TagExt,
    hb_font_t, hb_tag_t,
    ot_layout::LayoutLookup,
    ot_layout_gsubgpos::{Apply, WouldApply, WouldApplyContext, OT::hb_ot_apply_context_t},
    set_digest::hb_set_digest_ext,
};
use skrifa::raw::{
    tables::{
        gdef::Gdef,
        gpos::Gpos,
        gsub::{FeatureList, FeatureVariations, Gsub, ScriptList},
        layout::{ClassDef, Condition, CoverageTable, Feature, LangSys, Script},
        variations::ItemVariationStore,
    },
    types::{F2Dot14, Scalar},
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

pub enum LayoutTable<'a> {
    Gsub(Gsub<'a>),
    Gpos(Gpos<'a>),
}

impl<'a> LayoutTable<'a> {
    fn script_list(&self) -> Option<ScriptList<'a>> {
        match self {
            Self::Gsub(gsub) => gsub.script_list().ok(),
            Self::Gpos(gpos) => gpos.script_list().ok(),
        }
    }

    fn feature_list(&self) -> Option<FeatureList<'a>> {
        match self {
            Self::Gsub(gsub) => gsub.feature_list().ok(),
            Self::Gpos(gpos) => gpos.feature_list().ok(),
        }
    }

    fn feature_variations(&self) -> Option<FeatureVariations<'a>> {
        match self {
            Self::Gsub(gsub) => gsub.feature_variations(),
            Self::Gpos(gpos) => gpos.feature_variations(),
        }
        .transpose()
        .ok()
        .flatten()
    }

    fn script_index(&self, tag: hb_tag_t) -> Option<u16> {
        let list = self.script_list()?;
        list.script_records()
            .binary_search_by_key(&tag, |rec| rec.script_tag())
            .map(|index| index as u16)
            .ok()
    }

    fn script(&self, index: u16) -> Option<Script<'a>> {
        let list = self.script_list()?;
        let record = list.script_records().get(index as usize)?;
        record.script(list.offset_data()).ok()
    }

    fn langsys_index(&self, script_index: u16, tag: hb_tag_t) -> Option<u16> {
        let script = self.script(script_index)?;
        script
            .lang_sys_records()
            .binary_search_by_key(&tag, |rec| rec.lang_sys_tag())
            .map(|index| index as u16)
            .ok()
    }

    fn langsys(&self, script_index: u16, langsys_index: Option<u16>) -> Option<LangSys<'a>> {
        let script = self.script(script_index)?;
        if let Some(index) = langsys_index {
            let record = script.lang_sys_records().get(index as usize)?;
            record.lang_sys(script.offset_data()).ok()
        } else {
            script.default_lang_sys().transpose().ok().flatten()
        }
    }

    pub(crate) fn feature(&self, index: u16) -> Option<Feature<'a>> {
        let list = self.feature_list()?;
        let record = list.feature_records().get(index as usize)?;
        record.feature(list.offset_data()).ok()
    }

    fn feature_tag(&self, index: u16) -> Option<hb_tag_t> {
        let list = self.feature_list()?;
        let record = list.feature_records().get(index as usize)?;
        Some(record.feature_tag())
    }

    pub(crate) fn feature_variation_index(&self, coords: &[F2Dot14]) -> Option<u32> {
        let feature_variations = self.feature_variations()?;
        for (index, rec) in feature_variations
            .feature_variation_records()
            .iter()
            .enumerate()
        {
            // If the ConditionSet offset is 0, this is treated as the
            // universal condition: all contexts are matched.
            if rec.condition_set_offset().is_null() {
                return Some(index as u32);
            }
            let Some(Ok(condition_set)) = rec.condition_set(feature_variations.offset_data())
            else {
                continue;
            };
            // Otherwise, all conditions must be satisfied.
            if condition_set
                .conditions()
                .iter()
                // .. except we ignore errors
                .filter_map(|cond| cond.ok())
                .all(|cond| match cond {
                    Condition::Format1AxisRange(format1) => {
                        let coord = coords
                            .get(format1.axis_index() as usize)
                            .copied()
                            .unwrap_or_default();
                        coord >= format1.filter_range_min_value()
                            && coord <= format1.filter_range_max_value()
                    }
                    _ => false,
                })
            {
                return Some(index as u32);
            }
        }
        None
    }

    pub(crate) fn feature_substitution(
        &self,
        variation_index: u32,
        feature_index: u16,
    ) -> Option<Feature<'a>> {
        let feature_variations = self.feature_variations()?;
        let record = feature_variations
            .feature_variation_records()
            .get(variation_index as usize)?;
        let subst_table = record
            .feature_table_substitution(feature_variations.offset_data())?
            .ok()?;
        let subst_records = subst_table.substitutions();
        match subst_records.binary_search_by_key(&feature_index, |subst| subst.feature_index()) {
            Ok(ix) => Some(
                subst_records
                    .get(ix)?
                    .alternate_feature(subst_table.offset_data())
                    .ok()?,
            ),
            _ => None,
        }
    }

    pub(crate) fn feature_index(&self, tag: hb_tag_t) -> Option<u16> {
        let list = self.feature_list()?;
        for (index, feature) in list.feature_records().iter().enumerate() {
            if feature.feature_tag() == tag {
                return Some(index as u16);
            }
        }
        None
    }

    pub(crate) fn lookup_count(&self) -> u16 {
        match self {
            Self::Gsub(gsub) => gsub
                .lookup_list()
                .map(|list| list.lookup_count())
                .unwrap_or_default(),
            Self::Gpos(gpos) => gpos
                .lookup_list()
                .map(|list| list.lookup_count())
                .unwrap_or_default(),
        }
    }
}

impl crate::hb::ot_layout::LayoutTableExt for LayoutTable<'_> {
    // hb_ot_layout_table_select_script
    /// Returns true + index and tag of the first found script tag in the given GSUB or GPOS table
    /// or false + index and tag if falling back to a default script.
    fn select_script(&self, script_tags: &[hb_tag_t]) -> Option<(bool, u16, hb_tag_t)> {
        for &tag in script_tags {
            if let Some(index) = self.script_index(tag) {
                return Some((true, index, tag));
            }
        }

        for &tag in &[
            // try finding 'DFLT'
            hb_tag_t::default_script(),
            // try with 'dflt'; MS site has had typos and many fonts use it now :(
            hb_tag_t::default_language(),
            // try with 'latn'; some old fonts put their features there even though
            // they're really trying to support Thai, for example :(
            hb_tag_t::new(b"latn"),
        ] {
            if let Some(index) = self.script_index(tag) {
                return Some((false, index, tag));
            }
        }

        None
    }

    // hb_ot_layout_script_select_language
    /// Returns the index of the first found language tag in the given GSUB or GPOS table,
    /// underneath the specified script index.
    fn select_script_language(&self, script_index: u16, lang_tags: &[hb_tag_t]) -> Option<u16> {
        for &tag in lang_tags {
            if let Some(index) = self.langsys_index(script_index, tag) {
                return Some(index);
            }
        }

        // try finding 'dflt'
        if let Some(index) = self.langsys_index(script_index, hb_tag_t::default_language()) {
            return Some(index);
        }

        None
    }

    // hb_ot_layout_language_get_required_feature
    /// Returns the index and tag of a required feature in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    fn get_required_language_feature(
        &self,
        script_index: u16,
        lang_index: Option<u16>,
    ) -> Option<(u16, hb_tag_t)> {
        let sys = self.langsys(script_index, lang_index)?;
        let idx = sys.required_feature_index();
        if idx == 0xFFFF {
            return None;
        }
        let tag = self.feature_tag(idx)?;
        Some((idx, tag))
    }

    // hb_ot_layout_language_find_feature
    /// Returns the index of a given feature tag in the given GSUB or GPOS table,
    /// underneath the specified script and language.
    fn find_language_feature(
        &self,
        script_index: u16,
        lang_index: Option<u16>,
        feature_tag: hb_tag_t,
    ) -> Option<u16> {
        let sys = self.langsys(script_index, lang_index)?;
        for index in sys.feature_indices() {
            let index = index.get();
            if self.feature_tag(index) == Some(feature_tag) {
                return Some(index);
            }
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
