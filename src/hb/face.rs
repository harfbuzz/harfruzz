#[cfg(not(feature = "std"))]
use core_maths::CoreFloat;
use read_fonts::types::{F2Dot14, GlyphId};
use read_fonts::{FontRef, TableProvider};

use super::aat::AatTables;
use super::buffer::GlyphPropsFlags;
use super::charmap::Charmap;
use super::glyph_metrics::GlyphMetrics;
use super::glyph_names::GlyphNames;
use super::ot::{LayoutTable, OtCache, OtTables};
use super::ot_layout::TableIndex;

/// Cached shaper state for a single font.
pub struct ShaperFont {
    ot_cache: OtCache,
}

impl ShaperFont {
    /// Creates new cached state for the given font.
    pub fn new(font: &FontRef) -> Self {
        let ot_cache = OtCache::new(font);
        Self { ot_cache }
    }

    /// Creates a shaper instance for the given font and variation
    /// coordinates.
    pub fn shaper<'a>(&'a self, font: &FontRef<'a>, coords: &'a [F2Dot14]) -> Shaper<'a> {
        let units_per_em = font.head().map(|head| head.units_per_em()).unwrap_or(1000);
        let charmap = Charmap::new(font);
        let glyph_metrics = GlyphMetrics::new(font);
        let ot_tables = OtTables::new(font, &self.ot_cache, coords);
        let aat_tables = AatTables::new(font);
        hb_font_t {
            font: font.clone(),
            units_per_em,
            pixels_per_em: None,
            points_per_em: None,
            charmap,
            glyph_metrics,
            ot_tables,
            aat_tables,
        }
    }
}

/// A shaper.
pub type Shaper<'a> = hb_font_t<'a>;

/// A font face handle.
#[derive(Clone)]
pub struct hb_font_t<'a> {
    pub(crate) font: FontRef<'a>,
    pub(crate) units_per_em: u16,
    pixels_per_em: Option<(u16, u16)>,
    pub(crate) points_per_em: Option<f32>,
    charmap: Charmap<'a>,
    glyph_metrics: GlyphMetrics<'a>,
    pub(crate) ot_tables: OtTables<'a>,
    pub(crate) aat_tables: AatTables<'a>,
}

impl<'a> hb_font_t<'a> {
    // TODO: remove
    /// Returns face’s units per EM.
    #[inline]
    pub fn units_per_em(&self) -> i32 {
        self.units_per_em as i32
    }

    #[inline]
    pub(crate) fn pixels_per_em(&self) -> Option<(u16, u16)> {
        self.pixels_per_em
    }

    /// Sets pixels per EM.
    ///
    /// Used during raster glyphs processing and hinting.
    ///
    /// `None` by default.
    #[inline]
    pub fn set_pixels_per_em(&mut self, ppem: Option<(u16, u16)>) {
        self.pixels_per_em = ppem;
    }

    /// Sets point size per EM.
    ///
    /// Used for optical-sizing in Apple fonts.
    ///
    /// `None` by default.
    #[inline]
    pub fn set_points_per_em(&mut self, ptem: Option<f32>) {
        self.points_per_em = ptem;
    }

    pub(crate) fn has_glyph(&self, c: u32) -> bool {
        self.get_nominal_glyph(c).is_some()
    }

    pub(crate) fn get_nominal_glyph(&self, c: u32) -> Option<GlyphId> {
        self.charmap.map(c)
    }

    pub(crate) fn get_nominal_variant_glyph(&self, c: char, vs: char) -> Option<GlyphId> {
        self.charmap.map_variant(c as u32, vs as u32)
    }

    pub(crate) fn glyph_h_advance(&self, glyph: GlyphId) -> i32 {
        self.glyph_metrics
            .advance_width(glyph, self.ot_tables.coords)
            .unwrap_or_default()
    }

    pub(crate) fn glyph_v_advance(&self, glyph: GlyphId) -> i32 {
        -self
            .glyph_metrics
            .advance_height(glyph, self.ot_tables.coords)
            .unwrap_or(self.units_per_em as i32)
    }

    pub(crate) fn glyph_h_origin(&self, glyph: GlyphId) -> i32 {
        self.glyph_h_advance(glyph) / 2
    }

    pub(crate) fn glyph_v_origin(&self, glyph: GlyphId) -> i32 {
        self.glyph_metrics
            .v_origin(glyph, self.ot_tables.coords)
            .unwrap_or_default()
    }

    pub(crate) fn glyph_extents(
        &self,
        glyph: GlyphId,
        glyph_extents: &mut hb_glyph_extents_t,
    ) -> bool {
        if let Some(extents) = self.glyph_metrics.extents(glyph, self.ot_tables.coords) {
            glyph_extents.x_bearing = extents.x_min;
            glyph_extents.y_bearing = extents.y_max;
            glyph_extents.width = extents.x_max - extents.x_min;
            glyph_extents.height = extents.y_min - extents.y_max;
            true
        } else {
            false
        }
    }

    pub(crate) fn glyph_names(&self) -> GlyphNames<'a> {
        GlyphNames::new(&self.font)
    }

    pub(crate) fn glyph_props(&self, glyph: GlyphId) -> u16 {
        let glyph = glyph.to_u32();
        match self.ot_tables.glyph_class(glyph) {
            1 => GlyphPropsFlags::BASE_GLYPH.bits(),
            2 => GlyphPropsFlags::LIGATURE.bits(),
            3 => {
                let class = self.ot_tables.glyph_mark_attachment_class(glyph);
                (class << 8) | GlyphPropsFlags::MARK.bits()
            }
            _ => 0,
        }
    }

    pub(crate) fn layout_table(&self, table_index: TableIndex) -> Option<LayoutTable<'a>> {
        match table_index {
            TableIndex::GSUB => self
                .ot_tables
                .gsub
                .as_ref()
                .map(|table| LayoutTable::Gsub(table.table.clone())),
            TableIndex::GPOS => self
                .ot_tables
                .gpos
                .as_ref()
                .map(|table| LayoutTable::Gpos(table.table.clone())),
        }
    }

    pub(crate) fn layout_tables(&self) -> impl Iterator<Item = (TableIndex, LayoutTable<'a>)> + '_ {
        TableIndex::iter().filter_map(move |idx| self.layout_table(idx).map(|table| (idx, table)))
    }
}

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct hb_glyph_extents_t {
    pub x_bearing: i32,
    pub y_bearing: i32,
    pub width: i32,
    pub height: i32,
}
