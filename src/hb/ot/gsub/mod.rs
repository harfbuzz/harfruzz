//! OpenType GSUB lookups.

use super::lookup::{LookupCache, LookupInfo};
use crate::hb::ot_layout::TableIndex;
use read_fonts::{tables::gsub::Gsub, TableProvider};

mod alternate;
mod ligature;
mod multiple;
mod reverse_chain;
mod single;

#[derive(Clone)]
pub struct GsubTable<'a> {
    pub table: Gsub<'a>,
    pub lookups: LookupCache,
}

impl<'a> GsubTable<'a> {
    pub fn try_new(font: &impl TableProvider<'a>) -> Option<Self> {
        let table = font.gsub().ok()?;
        let mut lookups = LookupCache::new();
        lookups.create_all(&table);
        Some(Self { table, lookups })
    }
}

impl<'a> crate::hb::ot_layout::LayoutTable for GsubTable<'a> {
    const INDEX: TableIndex = TableIndex::GSUB;
    const IN_PLACE: bool = false;

    type Lookup = LookupInfo;

    fn get_lookup(&self, index: ttf_parser::opentype_layout::LookupIndex) -> Option<&Self::Lookup> {
        let lookup = self.lookups.get(index)?;
        if lookup.subtables_count == 0 {
            return None;
        }
        Some(lookup)
    }
}
