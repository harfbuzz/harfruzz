#[cfg(not(feature = "std"))]
use core_maths::CoreFloat;
use read_fonts::tables::trak::{TrackData, TrackTableEntry, Trak};
use read_fonts::types::{BigEndian, Fixed};

use super::buffer::hb_buffer_t;
use super::hb_font_t;
use super::ot_shape_plan::hb_ot_shape_plan_t;

pub fn apply(_plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) -> Option<()> {
    let trak = face.aat_tables.trak.as_ref()?;
    let mut ptem = face.points_per_em.unwrap_or(0.0) as f32;
    if ptem <= 0.0 {
        ptem = 12.0; // CoreText fallback
    }

    if !buffer.have_positions {
        buffer.clear_positions();
    }

    let advance_to_add = if buffer.direction.is_horizontal() {
        trak.get_h_tracking(ptem, 0.0)
    } else {
        trak.get_v_tracking(ptem, 0.0)
    };

    foreach_grapheme!(buffer, start, end, {
        if buffer.direction.is_horizontal() {
            buffer.pos[start].x_advance += advance_to_add;
        } else {
            buffer.pos[start].y_advance += advance_to_add;
        }
    });

    Some(())
}

trait TrakExt {
    fn get_h_tracking(&self, ptem: f32, track: f32) -> i32;
    fn get_v_tracking(&self, ptem: f32, track: f32) -> i32;
}

impl TrakExt for ttf_parser::trak::Table<'_> {
    fn get_h_tracking(&self, ptem: f32, track: f32) -> i32 {
        self.horizontal.get_tracking(ptem, track).round() as i32
    }

    fn get_v_tracking(&self, ptem: f32, track: f32) -> i32 {
        self.vertical.get_tracking(ptem, track).round() as i32
    }
}

trait TrackDataExt {
    fn get_tracking(&self, ptem: f32, track: f32) -> f32;
}

impl TrackDataExt for ttf_parser::trak::TrackData<'_> {
    fn get_tracking(&self, ptem: f32, track: f32) -> f32 {
        let sizes = &self.sizes;
        if sizes.is_empty() {
            return 0.0;
        }

        let tracks = &self.tracks;
        if tracks.is_empty() {
            return 0.0;
        }

        if tracks.len() == 1 {
            return tracks
                .get(0)
                .map(|t| t.get_value(ptem, sizes))
                .unwrap_or(0.0);
        }

        let mut i = 0;
        let mut j = tracks.len() - 1;

        while i + 1 < tracks.len() && tracks.get(i + 1).map(|t| t.value).unwrap_or(0.0) <= track {
            i += 1;
        }
        while j > 0 && tracks.get(j - 1).map(|t| t.value).unwrap_or(0.0) >= track {
            j -= 1;
        }

        if i == j {
            return tracks
                .get(i)
                .map(|t| t.get_value(ptem, sizes))
                .unwrap_or(0.0);
        }

        let t0 = tracks.get(i).map(|t| t.value).unwrap_or(0.0);
        let t1 = tracks.get(j).map(|t| t.value).unwrap_or(0.0);
        let interp = if (t1 - t0).abs() < f32::EPSILON {
            0.0
        } else {
            (track - t0) / (t1 - t0)
        };

        let a = tracks
            .get(i)
            .map(|t| t.get_value(ptem, sizes))
            .unwrap_or(0.0);
        let b = tracks
            .get(j)
            .map(|t| t.get_value(ptem, sizes))
            .unwrap_or(0.0);
        a + interp * (b - a)
    }
}

trait TrackEntryExt {
    fn get_value(&self, ptem: f32, sizes: &ttf_parser::LazyArray16<'_, ttf_parser::Fixed>) -> f32;
}

impl<'a> TrackEntryExt for ttf_parser::trak::Track<'a> {
    fn get_value(&self, ptem: f32, sizes: &ttf_parser::LazyArray16<'_, ttf_parser::Fixed>) -> f32 {
        let values = &self.values;
        let n = sizes.len().min(values.len());
        if n == 0 {
            return 0.0;
        }

        for i in 0..n {
            let size_pt = sizes.get(i).map(|f| f.0).unwrap_or(0.0);
            if size_pt >= ptem {
                if i == 0 {
                    return values.get(0).unwrap_or(0) as f32;
                }

                let s0 = sizes.get(i - 1).map(|f| f.0).unwrap_or(0.0);
                let s1 = size_pt;
                let v0 = values.get(i - 1).unwrap_or(0) as f32;
                let v1 = values.get(i).unwrap_or(0) as f32;

                if (s1 - s0).abs() < f32::EPSILON {
                    return (v0 + v1) * 0.5;
                }

                let t = (ptem - s0) / (s1 - s0);
                return v0 + t * (v1 - v0);
            }
        }

        values.get(n - 1).unwrap_or(0) as f32
    }
}
