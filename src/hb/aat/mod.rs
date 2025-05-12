use read_fonts::{
    tables::{ankr::Ankr, feat::Feat, kerx::Kerx, morx::Morx, trak::Trak},
    types::Tag,
    FontRef, TableProvider,
};
use ttf_parser::kern::Table as Kern;

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
        Self {
            morx: font.morx().ok(),
            ankr: font.ankr().ok(),
            kern: font
                .table_data(Tag::new(b"kern"))
                .and_then(|data| Kern::parse(data.as_bytes())),
            kerx: font.kerx().ok(),
            trak: font.trak().ok(),
            feat: font.feat().ok(),
        }
    }
}
