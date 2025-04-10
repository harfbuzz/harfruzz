#[cfg(not(feature = "std"))]
use core_maths::CoreFloat;
use read_fonts::tables::cff::Cff;
use read_fonts::tables::post::Post;
use read_fonts::tables::postscript::Charset;
use read_fonts::types::F2Dot14;
use read_fonts::{FontRef, TableProvider};

use crate::hb::paint_extents::hb_paint_extents_context_t;
use ttf_parser::{GlyphId, RgbaColor};

use super::aat::AatTables;
use super::buffer::GlyphPropsFlags;
use super::charmap::Charmap;
use super::common::TagExt;
use super::ot::{LayoutTable, OtFontCache, OtTables};
use super::ot_layout::TableIndex;
use crate::Variation;

pub struct ShaperFont {
    ot_cache: OtFontCache,
}

impl ShaperFont {
    pub fn new(font: &FontRef) -> Self {
        let ot_cache = OtFontCache::new(font);
        Self { ot_cache }
    }

    pub fn shaper<'a>(&'a self, font: &FontRef<'a>, coords: &'a [F2Dot14]) -> Shaper<'a> {
        let td_base = font.table_directory.offset_data().as_bytes().as_ptr();
        let face = if td_base != font.data.as_bytes().as_ptr() {
            let mut index = 0;
            for (i, font) in read_fonts::FileRef::new(font.data.as_bytes())
                .unwrap()
                .fonts()
                .enumerate()
            {
                if font
                    .unwrap()
                    .table_directory
                    .offset_data()
                    .as_bytes()
                    .as_ptr()
                    == td_base
                {
                    index = i;
                    break;
                }
            }
            ttf_parser::Face::parse(font.data.as_bytes(), index as u32).unwrap()
        } else {
            ttf_parser::Face::parse(font.data.as_bytes(), 0).unwrap()
        };
        let units_per_em = font.head().map(|head| head.units_per_em()).unwrap_or(1000);
        let charmap = Charmap::new(font);
        let glyph_names = GlyphNames::new(font);
        let ot_tables = OtTables::new(font, &self.ot_cache, coords);
        let aat_tables = AatTables::new(font);
        hb_font_t {
            font_ref: font.clone(),
            units_per_em,
            pixels_per_em: None,
            points_per_em: None,
            charmap,
            glyph_names,
            ot_tables,
            aat_tables,
            ttfp_face: face,
        }
    }
}

pub type Shaper<'a> = hb_font_t<'a>;

/// A font face handle.
#[derive(Clone)]
pub struct hb_font_t<'a> {
    pub(crate) ttfp_face: ttf_parser::Face<'a>,
    pub(crate) font_ref: FontRef<'a>,
    pub(crate) units_per_em: u16,
    pixels_per_em: Option<(u16, u16)>,
    pub(crate) points_per_em: Option<f32>,
    charmap: Charmap<'a>,
    glyph_names: Option<GlyphNames<'a>>,
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

    /// Sets font variations.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        for variation in variations {
            self.ttfp_face
                .set_variation(ttf_parser::Tag(variation.tag.as_u32()), variation.value);
        }
    }

    pub(crate) fn has_glyph(&self, c: u32) -> bool {
        self.get_nominal_glyph(c).is_some()
    }

    pub(crate) fn get_nominal_glyph(&self, c: u32) -> Option<GlyphId> {
        self.charmap.map(c).map(|gid| GlyphId(gid.to_u32() as u16))
    }

    pub(crate) fn get_nominal_variant_glyph(&self, c: char, vs: char) -> Option<GlyphId> {
        self.charmap
            .map_variant(c as u32, vs as u32)
            .map(|gid| GlyphId(gid.to_u32() as u16))
    }

    pub(crate) fn glyph_h_advance(&self, glyph: GlyphId) -> i32 {
        self.glyph_advance(glyph, false) as i32
    }

    pub(crate) fn glyph_v_advance(&self, glyph: GlyphId) -> i32 {
        -(self.glyph_advance(glyph, true) as i32)
    }

    fn glyph_advance(&self, glyph: GlyphId, is_vertical: bool) -> u32 {
        let face = &self.ttfp_face;
        if face.is_variable()
            && face.has_non_default_variation_coordinates()
            && face.tables().hvar.is_none()
            && face.tables().vvar.is_none()
            && face.glyph_phantom_points(glyph).is_none()
        {
            return match face.glyph_bounding_box(glyph) {
                Some(bbox) => {
                    (if is_vertical {
                        bbox.y_max + bbox.y_min
                    } else {
                        bbox.x_max + bbox.x_min
                    }) as u32
                }
                None => 0,
            };
        }

        if is_vertical {
            if face.tables().vmtx.is_some() {
                face.glyph_ver_advance(glyph).unwrap_or(0) as u32
            } else {
                // TODO: Original code calls `h_extents_with_fallback`
                (face.ascender() - face.descender()) as u32
            }
        } else if !is_vertical && face.tables().hmtx.is_some() {
            face.glyph_hor_advance(glyph).unwrap_or(0) as u32
        } else {
            face.units_per_em() as u32
        }
    }

    pub(crate) fn glyph_h_origin(&self, glyph: GlyphId) -> i32 {
        self.glyph_h_advance(glyph) / 2
    }

    pub(crate) fn glyph_v_origin(&self, glyph: GlyphId) -> i32 {
        match self.ttfp_face.glyph_y_origin(glyph) {
            Some(y) => i32::from(y),
            None => {
                let mut extents = hb_glyph_extents_t::default();
                if self.glyph_extents(glyph, &mut extents) {
                    if self.ttfp_face.tables().vmtx.is_some() {
                        extents.y_bearing + self.glyph_side_bearing(glyph, true)
                    } else {
                        let advance = self.ttfp_face.ascender() - self.ttfp_face.descender();
                        let diff = advance as i32 - -extents.height;
                        extents.y_bearing + (diff >> 1)
                    }
                } else {
                    // TODO: Original code calls `h_extents_with_fallback`
                    self.ttfp_face.ascender() as i32
                }
            }
        }
    }

    pub(crate) fn glyph_side_bearing(&self, glyph: GlyphId, is_vertical: bool) -> i32 {
        let face = &self.ttfp_face;
        if face.is_variable() && face.tables().hvar.is_none() && face.tables().vvar.is_none() {
            return match face.glyph_bounding_box(glyph) {
                Some(bbox) => (if is_vertical { bbox.x_min } else { bbox.y_min }) as i32,
                None => 0,
            };
        }

        if is_vertical {
            face.glyph_ver_side_bearing(glyph).unwrap_or(0) as i32
        } else {
            face.glyph_hor_side_bearing(glyph).unwrap_or(0) as i32
        }
    }

    pub(crate) fn glyph_extents(
        &self,
        glyph: GlyphId,
        glyph_extents: &mut hb_glyph_extents_t,
    ) -> bool {
        let pixels_per_em = self.pixels_per_em.map_or(u16::MAX, |ppem| ppem.0);

        if let Some(img) = self.ttfp_face.glyph_raster_image(glyph, pixels_per_em) {
            // HarfBuzz also supports only PNG.
            if img.format == ttf_parser::RasterImageFormat::PNG {
                let scale = self.units_per_em as f32 / img.pixels_per_em as f32;
                glyph_extents.x_bearing = (f32::from(img.x) * scale).round() as i32;
                glyph_extents.y_bearing =
                    ((f32::from(img.y) + f32::from(img.height)) * scale).round() as i32;
                glyph_extents.width = (f32::from(img.width) * scale).round() as i32;
                glyph_extents.height = (-f32::from(img.height) * scale).round() as i32;
                return true;
            }
        } else if let Some(colr) = self.ttfp_face.tables().colr {
            if let Some(clip_box) = colr.clip_box(glyph, self.ttfp_face.variation_coordinates()) {
                // Floor
                glyph_extents.x_bearing = (clip_box.x_min).round() as i32;
                glyph_extents.y_bearing = (clip_box.y_max).round() as i32;
                glyph_extents.width = (clip_box.x_max - clip_box.x_min).round() as i32;
                glyph_extents.height = (clip_box.y_min - clip_box.y_max).round() as i32;
                return true;
            }

            let mut extents_data = hb_paint_extents_context_t::new(&self.ttfp_face);
            let ret = colr
                .paint(
                    glyph,
                    0,
                    &mut extents_data,
                    self.ttfp_face.variation_coordinates(),
                    RgbaColor::new(0, 0, 0, 0),
                )
                .is_some();

            let e = extents_data.get_extents();
            if e.is_void() {
                glyph_extents.x_bearing = 0;
                glyph_extents.y_bearing = 0;
                glyph_extents.width = 0;
                glyph_extents.height = 0;
            } else {
                glyph_extents.x_bearing = e.x_min as i32;
                glyph_extents.y_bearing = e.y_max as i32;
                glyph_extents.width = (e.x_max - e.x_min) as i32;
                glyph_extents.height = (e.y_min - e.y_max) as i32;
            }

            if ret {
                return true;
            }
        }

        let mut bbox = None;

        if let Some(glyf) = self.ttfp_face.tables().glyf {
            bbox = glyf.bbox(glyph);
        }

        // See https://github.com/harfbuzz/rustybuzz/pull/98#issuecomment-1948430785
        if self.ttfp_face.tables().glyf.is_some() && bbox.is_none() {
            // Empty glyph; zero extents.
            return true;
        }

        let Some(bbox) = bbox else {
            return false;
        };

        glyph_extents.x_bearing = i32::from(bbox.x_min);
        glyph_extents.y_bearing = i32::from(bbox.y_max);
        glyph_extents.width = i32::from(bbox.width());
        glyph_extents.height = i32::from(bbox.y_min - bbox.y_max);

        true
    }

    pub(crate) fn glyph_name(&self, glyph: GlyphId) -> Option<&str> {
        let name = self.glyph_names.as_ref()?.get(glyph.0 as u32)?;
        (!name.is_empty()).then_some(name)
    }

    pub(crate) fn glyph_props(&self, glyph: GlyphId) -> u16 {
        let glyph = glyph.0 as u32;
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

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct hb_glyph_extents_t {
    pub x_bearing: i32,
    pub y_bearing: i32,
    pub width: i32,
    pub height: i32,
}

unsafe impl bytemuck::Zeroable for hb_glyph_extents_t {}
unsafe impl bytemuck::Pod for hb_glyph_extents_t {}

#[derive(Clone)]
enum GlyphNames<'a> {
    Cff(Cff<'a>, Charset<'a>),
    Post(Post<'a>),
}

impl<'a> GlyphNames<'a> {
    fn new(font: &FontRef<'a>) -> Option<Self> {
        if let Some((cff, charset)) = font
            .cff()
            .ok()
            .and_then(|cff| Some((cff.clone(), cff.charset(0).ok()??)))
        {
            Some(Self::Cff(cff, charset))
        } else if let Ok(post) = font.post() {
            Some(Self::Post(post))
        } else {
            None
        }
    }

    fn get(&self, glyph_id: u32) -> Option<&str> {
        match self {
            Self::Cff(cff, charset) => {
                let sid = charset.string_id(glyph_id.into()).ok()?;
                core::str::from_utf8(cff.string(sid)?.bytes()).ok()
            }
            Self::Post(post) => {
                let gid: u16 = glyph_id.try_into().ok()?;
                post.glyph_name(gid.into())
            }
        }
    }
}
