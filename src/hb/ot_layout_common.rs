#[allow(dead_code)]
pub mod lookup_flags {
    pub const RIGHT_TO_LEFT: u16 = 0x0001;
    pub const IGNORE_BASE_GLYPHS: u16 = 0x0002;
    pub const IGNORE_LIGATURES: u16 = 0x0004;
    pub const IGNORE_MARKS: u16 = 0x0008;
    pub const IGNORE_FLAGS: u16 = 0x000E;
    pub const USE_MARK_FILTERING_SET: u16 = 0x0010;
    pub const MARK_ATTACHMENT_TYPE_MASK: u16 = 0xFF00;
}

#[derive(Clone)]
pub struct PositioningTable<'a> {
    pub inner: ttf_parser::opentype_layout::LayoutTable<'a>,
}

impl<'a> PositioningTable<'a> {
    pub fn new(inner: ttf_parser::opentype_layout::LayoutTable<'a>) -> Self {
        Self { inner }
    }
}

#[derive(Clone)]
pub struct SubstitutionTable<'a> {
    pub inner: ttf_parser::opentype_layout::LayoutTable<'a>,
}

impl<'a> SubstitutionTable<'a> {
    pub fn new(inner: ttf_parser::opentype_layout::LayoutTable<'a>) -> Self {
        Self { inner }
    }
}
