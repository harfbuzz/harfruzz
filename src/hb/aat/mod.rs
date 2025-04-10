use core::num::NonZeroU16;
use read_fonts::{
    tables::{ankr::Ankr, feat::Feat},
    types::Tag,
    FontRef, TableProvider,
};
use ttf_parser::{
    kern::Table as Kern, kerx::Table as Kerx, morx::Table as Morx, trak::Table as Trak,
};

#[derive(Clone, Default)]
pub struct AatTables<'a> {
    pub morx: Option<Morx<'a>>,
    pub ankr: Option<Ankr<'a>>,
    pub kern: Option<Kern<'a>>,
    pub kerx: Option<Kerx<'a>>,
    pub trak: Option<Trak<'a>>,
    pub feat: Option<Feat<'a>>,
}

impl<'a> AatTables<'a> {
    pub fn new(font: &FontRef<'a>) -> Self {
        let Some(num_glyphs) = NonZeroU16::new(
            font.maxp()
                .map(|maxp| maxp.num_glyphs())
                .unwrap_or_default(),
        ) else {
            return Self::default();
        };
        Self {
            morx: font
                .table_data(Tag::new(b"morx"))
                .and_then(|data| Morx::parse(num_glyphs, data.as_bytes())),
            ankr: font.ankr().ok(),
            kern: font
                .table_data(Tag::new(b"kern"))
                .and_then(|data| Kern::parse(data.as_bytes())),
            kerx: font
                .table_data(Tag::new(b"kerx"))
                .and_then(|data| Kerx::parse(num_glyphs, data.as_bytes())),
            trak: font
                .table_data(Tag::new(b"trak"))
                .and_then(|data| Trak::parse(data.as_bytes())),
            feat: font.feat().ok(),
        }
    }
}
