use super::buffer::hb_buffer_t;
use super::Shaper;
use super::ot_layout::*;
use super::ot_shape_plan::hb_ot_shape_plan_t;

pub fn substitute(plan: &hb_ot_shape_plan_t, face: &Shaper, buffer: &mut hb_buffer_t) {
    apply_layout_table(plan, face, buffer, face.ot_tables.gsub.as_ref());
}
