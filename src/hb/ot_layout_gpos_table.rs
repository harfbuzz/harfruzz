#[cfg(not(feature = "std"))]
use core_maths::CoreFloat;

use super::buffer::*;
use super::hb_font_t;
use super::ot_layout::*;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use crate::Direction;

pub fn position(plan: &hb_ot_shape_plan_t, face: &hb_font_t, buffer: &mut hb_buffer_t) {
    apply_layout_table(plan, face, buffer, face.ot_tables.gpos.as_ref());
}

pub mod attach_type {
    pub const MARK: u8 = 1;
    pub const CURSIVE: u8 = 2;
}

/// Just like TryFrom<N>, but for numeric types not supported by the Rust's std.
pub(crate) trait TryNumFrom<T>: Sized {
    /// Casts between numeric types.
    fn try_num_from(_: T) -> Option<Self>;
}

impl TryNumFrom<f32> for i32 {
    #[inline]
    fn try_num_from(v: f32) -> Option<Self> {
        // Based on https://github.com/rust-num/num-traits/blob/master/src/cast.rs

        // Float as int truncates toward zero, so we want to allow values
        // in the exclusive range `(MIN-1, MAX+1)`.

        // We can't represent `MIN-1` exactly, but there's no fractional part
        // at this magnitude, so we can just use a `MIN` inclusive boundary.
        const MIN: f32 = i32::MIN as f32;
        // We can't represent `MAX` exactly, but it will round up to exactly
        // `MAX+1` (a power of two) when we cast it.
        const MAX_P1: f32 = i32::MAX as f32;
        if v >= MIN && v < MAX_P1 {
            Some(v as i32)
        } else {
            None
        }
    }
}

fn propagate_attachment_offsets(
    pos: &mut [GlyphPosition],
    len: usize,
    i: usize,
    direction: Direction,
) {
    // Adjusts offsets of attached glyphs (both cursive and mark) to accumulate
    // offset of glyph they are attached to.
    let chain = pos[i].attach_chain();
    let kind = pos[i].attach_type();
    if chain == 0 {
        return;
    }

    pos[i].set_attach_chain(0);

    let j = (i as isize + isize::from(chain)) as _;
    if j >= len {
        return;
    }

    propagate_attachment_offsets(pos, len, j, direction);

    match kind {
        attach_type::MARK => {
            pos[i].x_offset += pos[j].x_offset;
            pos[i].y_offset += pos[j].y_offset;

            assert!(j < i);
            if direction.is_forward() {
                for k in j..i {
                    pos[i].x_offset -= pos[k].x_advance;
                    pos[i].y_offset -= pos[k].y_advance;
                }
            } else {
                for k in j + 1..i + 1 {
                    pos[i].x_offset += pos[k].x_advance;
                    pos[i].y_offset += pos[k].y_advance;
                }
            }
        }
        attach_type::CURSIVE => {
            if direction.is_horizontal() {
                pos[i].y_offset += pos[j].y_offset;
            } else {
                pos[i].x_offset += pos[j].x_offset;
            }
        }
        _ => {}
    }
}

pub mod GPOS {
    use super::*;

    pub fn position_start(_: &hb_font_t, buffer: &mut hb_buffer_t) {
        let len = buffer.len;
        for pos in &mut buffer.pos[..len] {
            pos.set_attach_chain(0);
            pos.set_attach_type(0);
        }
    }

    pub fn position_finish_advances(_: &hb_font_t, _: &mut hb_buffer_t) {}

    pub fn position_finish_offsets(_: &hb_font_t, buffer: &mut hb_buffer_t) {
        let len = buffer.len;
        let direction = buffer.direction;

        // Handle attachments
        if buffer.scratch_flags & HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT != 0 {
            for i in 0..len {
                propagate_attachment_offsets(&mut buffer.pos, len, i, direction);
            }
        }
    }
}
