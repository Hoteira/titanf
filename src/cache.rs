#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::render::Metrics;
use crate::Map;
use crate::Vec;

pub struct Cache(Map<(u32, usize), (Metrics, Vec<u8>)>);

impl Cache {
    pub fn new() -> Self {
        Cache(Map::new())
    }

    #[inline(always)]
    pub fn flush(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    pub fn get(&self, id: u32, size: f32) -> Option<&(Metrics, Vec<u8>)> {
        self.0.get(&(id, size.ceil() as usize))
    }

    #[inline(always)]
    pub fn set(&mut self, size: u32, scale: f32, metrics: Metrics, data: Vec<u8>) {
        self.0.insert((size, scale.ceil() as usize), (metrics, data));
    }
}