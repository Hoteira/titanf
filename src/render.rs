#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::font::TrueTypeFont;
use crate::rasterizer::dda;
use crate::Vec;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub width: usize,
    pub height: usize,
    pub left_side_bearing: isize,
    pub advance_width: usize,
    pub base_line: isize,
}

impl TrueTypeFont {
    #[inline(always)]
    pub fn get_char<const CACHE: bool>(&mut self, c: char, size: f32) -> (Metrics, Vec<u8>) {
        let scale = size * (self.dpi / 72.0) / self.head.units_per_em as f32;
        let id = self.glyph_id_table.get(&c).unwrap_or(&0);

        if CACHE {
            let is_cached = self.cache.get(*id, size);
            if let Some(cached) = is_cached {
                return cached.clone();
            }
        }

        let glyph = self
            .glyph_data_table
            .get(&id)
            .unwrap_or(self.glyph_data_table.get(&0).unwrap());

        let width = (scale * glyph.bounds.width).ceil() as usize + 1;
        let height = (scale * glyph.bounds.height).ceil() as usize;
        let baseline = -(scale * glyph.y_max) as isize;

        let metrics = self.get_metrics(id, scale);
        let metrics = Metrics {
            width,
            height,
            advance_width: metrics.0,
            left_side_bearing: metrics.1,
            base_line: baseline,
        };

        let bitmap = dda::Rasterizer::new(width, height).draw(&glyph, scale).to_bitmap();
        if CACHE {
            self.cache.set(*id, size, metrics.clone(), bitmap.clone());
        }

        (metrics, bitmap)
    }
}