use skrifa::raw::tables::layout::CoverageTable;

/// Bloom filter for integer sets.
#[derive(Copy, Clone, Default)]
pub struct SetDigest(DigestInner);

impl SetDigest {
    /// Creates a new digest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts the given value.
    pub fn insert(&mut self, value: impl Into<u32>) {
        self.0.insert(value.into())
    }

    /// Inserts the given inclusive range.
    pub fn insert_range(&mut self, start: impl Into<u32>, end: impl Into<u32>) {
        self.0.insert_range(start.into(), end.into());
    }

    /// Inserts a collection of values.
    pub fn insert_all(&mut self, values: impl Iterator<Item = u32>) {
        self.0.insert_all(values)
    }

    /// Inserts all glyphs from a coverage table.
    pub fn insert_coverage(&mut self, coverage: &CoverageTable) {
        for gid in coverage.iter() {
            self.insert(gid.to_u16())
        }
    }

    /// Returns true if the filter may contain the given value.
    pub fn may_contain(&self, value: impl Into<u32>) -> bool {
        self.0.may_contain(value.into())
    }

    pub fn may_overlap(&self, other: &Self) -> bool {
        self.0.may_overlap(&other.0)
    }
}

impl core::fmt::Debug for SetDigest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SetDigest(..)")
    }
}

type DigestMask = u64;

trait DigestComponent: Copy {
    fn insert(&mut self, value: u32);
    fn insert_range(&mut self, a: u32, b: u32);
    fn insert_all(&mut self, values: impl Iterator<Item = u32>);
    fn may_contain(&self, value: u32) -> bool;
    fn may_overlap(&self, other: &Self) -> bool;
}

#[derive(Copy, Clone, Default, Debug)]
struct DigestPattern<const SHIFT: u32> {
    mask: DigestMask,
}

impl<const SHIFT: u32> DigestPattern<SHIFT> {
    const MASK_BITS: DigestMask = core::mem::size_of::<DigestMask>() as DigestMask * 8;

    fn mask_for(value: u32) -> u64 {
        let value = value as DigestMask;
        1 << ((value >> SHIFT) & (Self::MASK_BITS - 1))
    }
}

impl<const SHIFT: u32> DigestComponent for DigestPattern<SHIFT> {
    fn insert(&mut self, value: u32) {
        self.mask |= Self::mask_for(value)
    }

    fn insert_range(&mut self, a: u32, b: u32) {
        if ((b as DigestMask) >> SHIFT).wrapping_sub((a as DigestMask) >> SHIFT)
            >= Self::MASK_BITS - 1
        {
            self.mask = !0;
        } else {
            let ma = Self::mask_for(a);
            let mb = Self::mask_for(b);
            self.mask |= mb
                .wrapping_add(mb.wrapping_sub(ma))
                .wrapping_sub((mb < ma) as _);
        }
    }

    fn insert_all(&mut self, values: impl Iterator<Item = u32>) {
        for value in values {
            self.insert(value);
        }
    }

    fn may_contain(&self, value: u32) -> bool {
        (self.mask & Self::mask_for(value)) != 0
    }

    fn may_overlap(&self, other: &Self) -> bool {
        (self.mask & other.mask) != 0
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct DigestCombiner<H, T> {
    head: H,
    tail: T,
}

impl<H, T> DigestComponent for DigestCombiner<H, T>
where
    H: DigestComponent,
    T: DigestComponent,
{
    fn insert(&mut self, value: u32) {
        self.head.insert(value);
        self.tail.insert(value);
    }

    fn insert_range(&mut self, a: u32, b: u32) {
        self.head.insert_range(a, b);
        self.tail.insert_range(a, b);
    }

    fn insert_all(&mut self, values: impl Iterator<Item = u32>) {
        for value in values {
            self.head.insert(value);
            self.tail.insert(value);
        }
    }

    fn may_contain(&self, value: u32) -> bool {
        self.head.may_contain(value) && self.tail.may_contain(value)
    }

    fn may_overlap(&self, other: &Self) -> bool {
        self.head.may_overlap(&other.head) && self.tail.may_overlap(&other.tail)
    }
}

type DigestInner =
    DigestCombiner<DigestPattern<4>, DigestCombiner<DigestPattern<0>, DigestPattern<9>>>;
    