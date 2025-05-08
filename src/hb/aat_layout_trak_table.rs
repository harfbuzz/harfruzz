#[cfg(not(feature = "std"))]
use core_maths::CoreFloat;
use read_fonts::tables::trak::{TrackData, TrackTableEntry, Trak};
use read_fonts::types::{BigEndian, Fixed};

use super::buffer::hb_buffer_t;
use super::hb_font_t;
use super::ot_shape_plan::hb_ot_shape_plan_t;

pub fn apply(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) -> Option<()> {
    let trak_mask = plan.trak_mask;

    let ptem = face.points_per_em?;
    if ptem <= 0.0 {
        return None;
    }

    let trak = face.aat_tables.trak.as_ref()?;

    if !buffer.have_positions {
        buffer.clear_positions();
    }

    if buffer.direction.is_horizontal() {
        let tracking = trak
            .horiz()
            .transpose()
            .ok()
            .flatten()
            .and_then(|data| Tracker::new(trak, data)?.tracking(ptem))?;
        let advance_to_add = tracking;
        let offset_to_add = tracking / 2;
        foreach_grapheme!(buffer, start, end, {
            if buffer.info[start].mask & trak_mask != 0 {
                buffer.pos[start].x_advance += advance_to_add;
                buffer.pos[start].x_offset += offset_to_add;
            }
        });
    } else {
        let tracking = trak
            .vert()
            .transpose()
            .ok()
            .flatten()
            .and_then(|data| Tracker::new(trak, data)?.tracking(ptem))?;
        let advance_to_add = tracking;
        let offset_to_add = tracking / 2;
        foreach_grapheme!(buffer, start, end, {
            if buffer.info[start].mask & trak_mask != 0 {
                buffer.pos[start].y_advance += advance_to_add;
                buffer.pos[start].y_offset += offset_to_add;
            }
        });
    }

    Some(())
}

struct Tracker<'a> {
    trak: &'a Trak<'a>,
    sizes: &'a [BigEndian<Fixed>],
    tracks: &'a [TrackTableEntry],
}

impl<'a> Tracker<'a> {
    fn new(trak: &'a Trak<'a>, data: TrackData<'a>) -> Option<Self> {
        let sizes = data.size_table(trak.offset_data()).ok()?;
        let tracks = data.track_table();
        Some(Self {
            trak,
            sizes,
            tracks,
        })
    }

    fn tracking(&self, ptem: f32) -> Option<i32> {
        // Choose track.
        let track = self.tracks.into_iter().find(|t| t.track() == Fixed::ZERO)?;

        // Choose size.
        if self.sizes.is_empty() {
            return None;
        }

        let mut idx = self
            .sizes
            .into_iter()
            .position(|s| s.get().to_f32() >= ptem)
            .unwrap_or(self.sizes.len() as usize - 1);

        idx = idx.saturating_sub(1);

        self.interpolate_at(
            idx,
            ptem,
            track
                .per_size_values(self.trak.offset_data(), self.sizes.len() as u16)
                .ok()?,
        )
        .map(|n| n.round() as i32)
    }

    fn interpolate_at(
        &self,
        idx: usize,
        target_size: f32,
        values: &[BigEndian<i16>],
    ) -> Option<f32> {
        debug_assert!(idx < self.sizes.len() - 1);

        let s0 = self.sizes.get(idx)?.get().to_f32();
        let s1 = self.sizes.get(idx + 1)?.get().to_f32();

        let t = if s0 == s1 {
            0.0
        } else {
            (target_size - s0) / (s1 - s0)
        };
        let n =
            t * (values.get(idx + 1)?.get() as f32) + (1.0 - t) * (values.get(idx)?.get() as f32);

        Some(n)
    }
}
