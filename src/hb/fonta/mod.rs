pub mod ot;
mod set_digest;

use alloc::vec::Vec;
use set_digest::SetDigest;
use skrifa::{
    charmap::Charmap,
    instance::NormalizedCoord,
    raw::{
        tables::{
            gpos::{AnchorTable, DeviceOrVariationIndex},
            variations::{DeltaSetIndex, ItemVariationStore},
        },
        ReadError,
    },
    MetadataProvider,
};
use ttf_parser::NormalizedCoordinate;

#[derive(Clone)]
pub struct Font<'a> {
    pub charmap: Charmap<'a>,
    pub ot: ot::LayoutTables<'a>,
    pub coords: Vec<NormalizedCoord>,
    pub ivs: Option<ItemVariationStore<'a>>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8], font_index: u32) -> Option<Self> {
        let font = skrifa::FontRef::from_index(data, font_index).ok()?;
        let charmap = font.charmap();
        let ot = ot::LayoutTables::new(&font);
        Some(Self {
            charmap,
            ot,
            coords: Vec::new(),
            ivs: None,
        })
    }

    pub(crate) fn set_coords(&mut self, coords: &[NormalizedCoordinate]) {
        self.coords.clear();
        if !coords.is_empty() && !coords.iter().all(|coord| coord.get() == 0) {
            let ivs = self.ivs.take().or_else(|| self.ot.item_variation_store());
            if ivs.is_some() {
                self.coords.extend(
                    coords
                        .iter()
                        .map(|coord| NormalizedCoord::from_bits(coord.get())),
                );
            }
            self.ivs = ivs;
        } else {
            self.ivs = None;
        }
    }

    fn resolve_anchor(&self, anchor: &AnchorTable) -> (i32, i32) {
        let mut x = anchor.x_coordinate() as i32;
        let mut y = anchor.y_coordinate() as i32;
        if let Some(ivs) = self.ivs.as_ref() {
            let delta = |val: Option<Result<DeviceOrVariationIndex<'_>, ReadError>>| match val {
                Some(Ok(DeviceOrVariationIndex::VariationIndex(varix))) => ivs
                    .compute_delta(
                        DeltaSetIndex {
                            outer: varix.delta_set_outer_index(),
                            inner: varix.delta_set_inner_index(),
                        },
                        &self.coords,
                    )
                    .unwrap_or_default(),
                _ => 0,
            };
            x += delta(anchor.x_device());
            y += delta(anchor.y_device());
        }
        (x, y)
    }
}
