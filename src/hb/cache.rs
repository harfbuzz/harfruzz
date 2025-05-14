use core::marker::PhantomData;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

/// Trait for atomics used in cache storage
pub trait AtomicStorage: Default {
    const BITS: usize;
    fn get(&self) -> u32;
    fn set(&self, val: u32);
}

/// AtomicU16 wrapper
#[derive(Debug)]
pub struct Atomic16(AtomicU16);
impl Default for Atomic16 {
    fn default() -> Self {
        Self(AtomicU16::new(u16::MAX))
    }
}
impl AtomicStorage for Atomic16 {
    const BITS: usize = 16;

    fn get(&self) -> u32 {
        self.0.load(Ordering::Relaxed) as u32
    }
    fn set(&self, val: u32) {
        self.0.store(val as u16, Ordering::Relaxed)
    }
}

/// AtomicU32 wrapper
#[derive(Debug)]
pub struct Atomic32(AtomicU32);
impl Default for Atomic32 {
    fn default() -> Self {
        Self(AtomicU32::new(u32::MAX))
    }
}
impl AtomicStorage for Atomic32 {
    const BITS: usize = 32;

    fn get(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }
    fn set(&self, val: u32) {
        self.0.store(val, Ordering::Relaxed)
    }
}

/// Selects correct wrapper type from STORAGE_BITS
pub trait SelectAtomic<const BITS: usize> {
    type Type: AtomicStorage;
}
impl SelectAtomic<16> for () {
    type Type = Atomic16;
}
impl SelectAtomic<32> for () {
    type Type = Atomic32;
}

/// Public wrapper
pub type hb_cache_t<
    const KEY_BITS: usize,
    const VALUE_BITS: usize,
    const CACHE_SIZE: usize,
    const STORAGE_BITS: usize,
> = hb_cache_core_t<KEY_BITS, VALUE_BITS, CACHE_SIZE, <() as SelectAtomic<STORAGE_BITS>>::Type>;

/// Core cache
#[derive(Debug)]
pub struct hb_cache_core_t<
    const KEY_BITS: usize,
    const VALUE_BITS: usize,
    const CACHE_SIZE: usize,
    T: AtomicStorage,
> {
    values: [T; CACHE_SIZE],
    _marker: PhantomData<T>,
}

impl<const KEY_BITS: usize, const VALUE_BITS: usize, const CACHE_SIZE: usize, T: AtomicStorage>
    Default for hb_cache_core_t<KEY_BITS, VALUE_BITS, CACHE_SIZE, T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const KEY_BITS: usize, const VALUE_BITS: usize, const CACHE_SIZE: usize, T: AtomicStorage>
    hb_cache_core_t<KEY_BITS, VALUE_BITS, CACHE_SIZE, T>
{
    pub fn new() -> Self {
        debug_assert!(
            CACHE_SIZE.is_power_of_two(),
            "CACHE_SIZE must be a power of two"
        );

        let cache_bits = CACHE_SIZE.ilog2() as usize;

        debug_assert!(
            KEY_BITS >= cache_bits,
            "KEY_BITS must be >= log2(CACHE_SIZE)"
        );
        debug_assert!(
            KEY_BITS + VALUE_BITS <= cache_bits + T::BITS,
            "KEY_BITS + VALUE_BITS must fit in CACHE_BITS + T::BITS"
        );

        Self {
            values: core::array::from_fn(|_| T::default()),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn get(&self, key: u32) -> Option<u32> {
        let index = (key as usize) & (CACHE_SIZE - 1);
        let stored = T::get(&self.values[index]);
        let tag = stored >> VALUE_BITS;
        let expected_tag = key >> (CACHE_SIZE as u32).ilog2();

        if stored == u32::MAX || tag != expected_tag {
            return None;
        }

        Some(stored & ((1 << VALUE_BITS) - 1))
    }

    #[inline]
    pub fn set(&self, key: u32, value: u32) -> bool {
        if (key >> KEY_BITS) != 0 || (value >> VALUE_BITS) != 0 {
            return false;
        }

        let index = (key as usize) & (CACHE_SIZE - 1);
        let packed = ((key >> (CACHE_SIZE as u32).ilog2()) << VALUE_BITS) | value;
        T::set(&self.values[index], packed);
        true
    }
}
