//! Common tables for OpenType layout.

use std::cmp::Ordering;
use std::convert::TryFrom;

use ttf_parser::parser::{FromData, LazyArray16, Offset16, Offset32, Offsets16, Stream, TryNumFrom};
use ttf_parser::GlyphId;

use crate::font::Font;

/// A type-safe wrapper for a glyph class.
#[repr(transparent)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct GlyphClass(pub u16);

impl FromData for GlyphClass {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

/// A GSUB or GPOS table.
#[derive(Clone, Copy)]
pub struct SubstPosTable<'a> {
    lookups: LookupList<'a>,
}

impl<'a> SubstPosTable<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let major_version = s.read::<u16>()?;
        let minor_version = s.read::<u16>()?;
        if major_version != 1 {
            return None;
        }

        s.skip::<Offset16>(); // TODO: script list
        s.skip::<Offset16>(); // TODO: feature list
        let lookups = LookupList::parse(s.read_offset16_data()?)?;
        if minor_version >= 1 {
            s.skip::<Offset32>(); // TODO: feature variations
        }

        Some(Self { lookups })
    }

    pub fn lookups(&self) -> LookupList {
        self.lookups
    }
}

#[derive(Clone, Copy)]
pub struct LookupList<'a> {
    offsets: Offsets16<'a, Offset16>,
}

impl<'a> LookupList<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;
        Some(LookupList { offsets })
    }

    pub fn len(&self) -> usize {
        usize::from(self.offsets.len())
    }

    pub fn get(&self, index: usize) -> Option<Lookup<'a>> {
        Lookup::parse(self.offsets.slice(u16::try_from(index).ok()?)?)
    }
}

#[derive(Clone, Copy)]
pub struct Lookup<'a> {
    pub type_: u16,
    pub flags: LookupFlags,
    pub offsets: Offsets16<'a, Offset16>,
    pub mark_filtering_set: Option<u16>,
}

impl<'a> Lookup<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let type_ = s.read::<u16>()?;
        let flags = s.read::<LookupFlags>()?;
        let count = s.read::<u16>()?;
        let offsets = s.read_offsets16(count, data)?;

        let mut mark_filtering_set: Option<u16> = None;
        if flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
            mark_filtering_set = Some(s.read()?);
        }

        Some(Self {
            type_,
            flags,
            offsets,
            mark_filtering_set,
        })
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct LookupFlags: u16 {
        const RIGHT_TO_LEFT          = 0x0001;
        const IGNORE_BASE_GLYPHS     = 0x0002;
        const IGNORE_LIGATURES       = 0x0004;
        const IGNORE_MARKS           = 0x0008;
        const IGNORE_FLAGS           = 0x000E;
        const USE_MARK_FILTERING_SET = 0x0010;
        const MARK_ATTACHMENT_TYPE   = 0xFF00;
    }
}

impl FromData for LookupFlags {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self::from_bits_truncate)
    }
}

/// A record that describes a range of glyph ids.
#[derive(Clone, Copy, Debug)]
pub struct RangeRecord {
    start: GlyphId,
    end: GlyphId,
    value: u16,
}

impl RangeRecord {
    fn binary_search(records: &LazyArray16<RangeRecord>, glyph: GlyphId) -> Option<RangeRecord> {
        records.binary_search_by(|record| {
            if glyph < record.start {
                Ordering::Greater
            } else if glyph <= record.end {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }).map(|p| p.1)
    }
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            start: s.read::<GlyphId>()?,
            end: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}

/// A table that defines which glyph ids are covered by some lookup.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table
#[derive(Clone, Copy, Debug)]
pub enum Coverage<'a> {
    Format1 { glyphs: LazyArray16<'a, GlyphId> },
    Format2 { records: LazyArray16<'a, RangeRecord> },
}

impl<'a> Coverage<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let count = s.read::<u16>()?;
                let glyphs = s.read_array16(count)?;
                Self::Format1 { glyphs }
            }
            2 => {
                let count = s.read::<u16>()?;
                let records = s.read_array16(count)?;
                Self::Format2 { records }
            }
            _ => return None,
        })
    }

    /// Returns the coverage index of the glyph or `None` if it is not covered.
    pub fn get(&self, glyph: GlyphId) -> Option<u16> {
        match self {
            Self::Format1 { glyphs } => glyphs.binary_search(&glyph).map(|p| p.0),
            Self::Format2 { records } => {
                let record = RangeRecord::binary_search(records, glyph)?;
                let offset = glyph.0 - record.start.0;
                record.value.checked_add(offset)
            }
        }
    }
}

/// A table that defines which classes glyph ids belong to.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table
#[derive(Clone, Copy, Debug)]
pub enum ClassDef<'a> {
    Format1 {
        start: GlyphId,
        classes: LazyArray16<'a, GlyphClass>,
    },
    Format2 {
        records: LazyArray16<'a, RangeRecord>,
    },
}

impl<'a> ClassDef<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        Some(match format {
            1 => {
                let start = s.read::<GlyphId>()?;
                let count = s.read::<u16>()?;
                let classes = s.read_array16(count)?;
                Self::Format1 { start, classes }
            },
            2 => {
                let count = s.read::<u16>()?;
                Self::Format2 { records: s.read_array16(count)? }
            },
            _ => return None,
        })
    }

    /// Returns the glyph class of the glyph (zero if it is not defined).
    pub fn get(&self, glyph: GlyphId) -> GlyphClass {
        let class = match self {
            Self::Format1 { start, classes } => {
                glyph.0.checked_sub(start.0)
                    .and_then(|index| classes.get(index))
            }
            Self::Format2 { records } => {
                RangeRecord::binary_search(records, glyph)
                    .map(|record| GlyphClass(record.value))
            }
        };
        class.unwrap_or(GlyphClass(0))
    }
}
