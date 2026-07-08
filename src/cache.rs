use crate::Vec;
use crate::fastmap::FastMap;
use crate::render::Metrics;

const MAX_ENTRIES: usize = 4096;

/// Bitmap cache keyed by glyph id and exact `f32` size bit pattern.
/// Backed by an open-addressing FastMap for O(1) lookups.
/// Bounded to MAX_ENTRIES: flushes wholesale when full (no LRU overhead).
pub struct Cache(FastMap<(Metrics, Vec<u8>)>);

#[inline(always)]
fn key(id: u32, size: f32) -> u64 {
    ((id as u64) << 32) | size.to_bits() as u64
}

impl Cache {
    pub fn new() -> Self {
        Cache(FastMap::new())
    }

    pub fn flush(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn get(&self, id: u32, size: f32) -> Option<&(Metrics, Vec<u8>)> {
        self.0.get(key(id, size))
    }

    pub fn set(&mut self, id: u32, size: f32, metrics: Metrics, data: Vec<u8>) {
        if self.0.len() >= MAX_ENTRIES {
            self.0.clear();
        }
        self.0.insert(key(id, size), (metrics, data));
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
