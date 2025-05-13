use read_fonts::{
    tables::{
        glyf::Glyf, gvar::Gvar, hmtx::Hmtx, hvar::Hvar, loca::Loca, vmtx::Vmtx, vorg::Vorg,
        vvar::Vvar,
    },
    types::{BoundingBox, F2Dot14, Fixed, GlyphId, Point},
    FontRef, TableProvider,
};

#[derive(Clone)]
pub(crate) struct GlyphMetrics<'a> {
    hmtx: Option<Hmtx<'a>>,
    hvar: Option<Hvar<'a>>,
    vmtx: Option<Vmtx<'a>>,
    vvar: Option<Vvar<'a>>,
    vorg: Option<Vorg<'a>>,
    glyf: Option<GlyfTables<'a>>,
    ascent: i16,
    descent: i16,
}

#[derive(Clone)]
struct GlyfTables<'a> {
    loca: Loca<'a>,
    glyf: Glyf<'a>,
    gvar: Option<Gvar<'a>>,
}

impl<'a> GlyphMetrics<'a> {
    pub fn new(font: &FontRef<'a>) -> Self {
        let hmtx = font.hmtx().ok();
        let hvar = font.hvar().ok();
        let vmtx = font.vmtx().ok();
        let vvar = font.vvar().ok();
        let vorg = font.vorg().ok();
        let glyf = if let (Ok(glyf), Ok(loca)) = (font.glyf(), font.loca(None)) {
            Some(GlyfTables {
                glyf,
                loca,
                gvar: font.gvar().ok(),
            })
        } else {
            None
        };
        let (ascent, descent) = if let Ok(os2) = font.os2() {
            (os2.s_typo_ascender(), os2.s_typo_descender())
        } else if let Ok(hhea) = font.hhea() {
            (hhea.ascender().to_i16(), hhea.descender().to_i16())
        } else {
            (0, 0) // TODO
        };
        Self {
            hmtx,
            hvar,
            vmtx,
            vvar,
            vorg,
            glyf,
            ascent,
            descent,
        }
    }

    pub fn advance_width(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<i32> {
        let gid = gid.into();
        let mut advance = if let Some(hmtx) = self.hmtx.as_ref() {
            hmtx.advance(gid)? as i32
        } else if let Some(extents) = self.extents(gid, coords) {
            return Some(extents.x_max + extents.x_min);
        } else {
            return None;
        };
        if !coords.is_empty() {
            if let Some(hvar) = self.hvar.as_ref() {
                advance += hvar
                    .advance_width_delta(gid, coords)
                    .unwrap_or_default()
                    .to_i32();
            } else if let Some(deltas) = self.phantom_deltas(gid, coords) {
                advance += deltas[1].x.to_i32() - deltas[0].x.to_i32();
            }
        }
        Some(advance)
    }

    pub fn _left_side_bearing(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<i32> {
        let gid = gid.into();
        let mut bearing = if let Some(hmtx) = self.hmtx.as_ref() {
            hmtx.side_bearing(gid).unwrap_or_default() as i32
        } else if let Some(extents) = self.extents(gid, coords) {
            return Some(extents.x_min);
        } else {
            return None;
        };
        if !coords.is_empty() {
            if let Some(hvar) = self.hvar.as_ref() {
                bearing += hvar.lsb_delta(gid, coords).unwrap_or_default().to_i32();
            } else if let Some(deltas) = self.phantom_deltas(gid, coords) {
                bearing += deltas[0].x.to_i32();
            }
        }
        Some(bearing)
    }

    pub fn advance_height(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<i32> {
        let gid = gid.into();
        let mut advance = if let Some(vmtx) = self.vmtx.as_ref() {
            vmtx.advance(gid).unwrap_or(0) as i32
        } else {
            self.ascent as i32 - self.descent as i32
        };
        if !coords.is_empty() {
            if let Some(vvar) = self.vvar.as_ref() {
                advance += vvar
                    .advance_height_delta(gid, coords)
                    .unwrap_or_default()
                    .to_i32();
            } else if let Some(deltas) = self.phantom_deltas(gid, coords) {
                advance += deltas[3].y.to_i32() - deltas[2].y.to_i32();
            }
        }
        Some(advance)
    }

    pub fn top_side_bearing(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<i32> {
        let gid = gid.into();
        let mut bearing = if let Some(vmtx) = self.vmtx.as_ref() {
            vmtx.side_bearing(gid).unwrap_or_default() as i32
        } else {
            return None;
        };
        if !coords.is_empty() {
            if let Some(vvar) = self.vvar.as_ref() {
                bearing += vvar.tsb_delta(gid, coords).unwrap_or_default().to_i32();
            } else if let Some(deltas) = self.phantom_deltas(gid, coords) {
                bearing += deltas[3].y.to_i32();
            }
        }
        Some(bearing)
    }

    pub fn v_origin(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<i32> {
        let gid = gid.into();
        let origin = if let Some(vorg) = self.vorg.as_ref() {
            let mut origin = vorg.vertical_origin_y(gid) as i32;
            if !coords.is_empty() {
                if let Some(vvar) = self.vvar.as_ref() {
                    origin += vvar
                        .v_org_delta(gid, coords)
                        .unwrap_or_default()
                        .round()
                        .to_i32();
                }
            }
            origin
        } else if let Some(extents) = self.extents(gid, coords) {
            let origin = if self.vmtx.is_some() {
                let mut origin = Some(extents.y_max);
                let tsb = self.top_side_bearing(gid, coords);
                if let Some(tsb) = tsb {
                    origin = Some(origin.unwrap() + tsb);
                } else {
                    origin = None;
                }
                if origin.is_some() && !coords.is_empty() {
                    if let Some(vvar) = self.vvar.as_ref() {
                        origin = Some(
                            origin.unwrap()
                                + vvar
                                    .v_org_delta(gid, coords)
                                    .unwrap_or_default()
                                    .round()
                                    .to_i32(),
                        );
                    }
                }
                origin
            } else {
                None
            };

            if let Some(origin) = origin {
                origin
            } else {
                let advance = self.ascent as i32 - self.descent as i32;
                let diff = advance - (extents.y_max - extents.y_min);
                extents.y_max + (diff >> 1)
            }
        } else {
            self.ascent as i32
        };
        Some(origin)
    }

    pub fn extents(&self, gid: impl Into<GlyphId>, coords: &[F2Dot14]) -> Option<BoundingBox<i32>> {
        let gid = gid.into();
        let glyf = self.glyf.as_ref()?;
        let glyph = glyf.loca.get_glyf(gid, &glyf.glyf).ok()?;
        let Some(glyph) = glyph else {
            // Return empty extents for empty glyph
            return Some(BoundingBox::default());
        };
        let mut bbox = BoundingBox {
            x_min: glyph.x_min() as i32,
            x_max: glyph.x_max() as i32,
            y_min: glyph.y_min() as i32,
            y_max: glyph.y_max() as i32,
        };
        if !coords.is_empty() {
            return None;
        }
        Some(bbox)
    }

    fn phantom_deltas(&self, gid: GlyphId, coords: &[F2Dot14]) -> Option<[Point<Fixed>; 4]> {
        let glyf = self.glyf.as_ref()?;
        let gvar = glyf.gvar.as_ref()?;
        gvar.phantom_point_deltas(&glyf.glyf, &glyf.loca, coords, gid)
            .ok()?
    }
}
