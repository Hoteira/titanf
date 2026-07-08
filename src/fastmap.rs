//! Minimal open-addressing hash map for the render hot path.
//!
//! The default `no_std` build previously fell back to `BTreeMap`, putting an
//! O(log n) pointer chase in front of every `get_char`. This map does one
//! multiply (Fibonacci hashing) and a short linear probe instead, works in
//! `no_std`, and adds no dependencies. Keys are `u64`; `u64::MAX` is reserved
//! as the empty sentinel (no valid key uses it: chars are <= 0x10FFFF and
//! cache keys are (id << 32) | size_bits with id well below u32::MAX).

use crate::Vec;
use crate::vec;

const EMPTY: u64 = u64::MAX;
const FIB: u64 = 0x9E37_79B9_7F4A_7C15;

pub(crate) struct FastMap<V> {
    keys: Vec<u64>,
    values: Vec<Option<V>>,
    mask: usize,
    len: usize,
}

impl<V> FastMap<V> {
    pub(crate) fn new() -> Self {
        Self::with_pow2_capacity(16)
    }

    fn with_pow2_capacity(cap: usize) -> Self {
        debug_assert!(cap.is_power_of_two());
        let mut values = Vec::new();
        values.resize_with(cap, || None);
        FastMap {
            keys: vec![EMPTY; cap],
            values,
            mask: cap - 1,
            len: 0,
        }
    }

    #[inline(always)]
    fn slot(&self, key: u64) -> usize {
        // Fibonacci hashing: one multiply, top bits selected by the mask
        // width. Excellent spread for small integer keys.
        let h = key.wrapping_mul(FIB);
        (h >> (64 - (self.mask + 1).trailing_zeros())) as usize & self.mask
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn clear(&mut self) {
        self.keys.iter_mut().for_each(|k| *k = EMPTY);
        self.values.iter_mut().for_each(|v| *v = None);
        self.len = 0;
    }

    #[inline(always)]
    pub(crate) fn get(&self, key: u64) -> Option<&V> {
        debug_assert!(key != EMPTY);
        let mut i = self.slot(key);
        loop {
            let k = unsafe { *self.keys.get_unchecked(i) };
            if k == key {
                return unsafe { self.values.get_unchecked(i) }.as_ref();
            }
            if k == EMPTY {
                return None;
            }
            i = (i + 1) & self.mask;
        }
    }

    pub(crate) fn insert(&mut self, key: u64, value: V) {
        debug_assert!(key != EMPTY);
        // Keep load factor under 75%.
        if (self.len + 1) * 4 > (self.mask + 1) * 3 {
            self.grow();
        }
        let mut i = self.slot(key);
        loop {
            let k = self.keys[i];
            if k == key {
                self.values[i] = Some(value);
                return;
            }
            if k == EMPTY {
                self.keys[i] = key;
                self.values[i] = Some(value);
                self.len += 1;
                return;
            }
            i = (i + 1) & self.mask;
        }
    }

    fn grow(&mut self) {
        let new_cap = (self.mask + 1) * 2;
        let mut next = Self::with_pow2_capacity(new_cap);
        for (i, &k) in self.keys.iter().enumerate() {
            if k != EMPTY
                && let Some(v) = self.values[i].take() {
                    next.insert(k, v);
                }
        }
        *self = next;
    }
}
