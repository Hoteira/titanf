#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::font::TrueTypeFont;
use crate::geometry::lines::Line;
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
            .get(*id as usize)
            .and_then(|opt| opt.as_ref())
            .or_else(|| self.glyph_data_table.get(0).and_then(|opt| opt.as_ref()))
            .expect("Glyph 0 missing");

        glyph.build_lines_into::<false>(
            self.head.units_per_em as f32,
            scale,
            &mut self.lines_scratch,
            &mut self.segments_scratch,
        );

        let width = (scale * self.lines_scratch.bounds.width).ceil() as usize;
        let height = (scale * self.lines_scratch.bounds.height).ceil() as usize;
        let baseline = -(scale * glyph.y_max) as isize;

        let metrics = self.get_metrics(id, scale);
        let metrics = Metrics {
            width,
            height,
            advance_width: metrics.0,
            left_side_bearing: metrics.1,
            base_line: baseline,
        };


        self.rasterizer.reset(width, height);
        self.rasterizer.set_dirty_region(0.0, height as f32); // Use calculated height for bounds
        let bitmap = self.rasterizer.draw(&self.lines_scratch.v_lines, &self.lines_scratch.m_lines).to_bitmap();
        if CACHE {
            self.cache.set(*id, size, metrics.clone(), bitmap.clone());
        }

        (metrics, bitmap)
    }

    pub fn get_char_lines(&self, c: char, size: f32) -> (Metrics, Vec<Line>, Vec<Line>, Vec<Line>) {

        let scale = size * (self.dpi / 72.0) / self.head.units_per_em as f32;
        let id = self.glyph_id_table.get(&c).unwrap_or(&0);

        let glyph = self
            .glyph_data_table
            .get(*id as usize)
            .and_then(|opt| opt.as_ref())
            .or_else(|| self.glyph_data_table.get(0).and_then(|opt| opt.as_ref()))
            .expect("Glyph 0 missing");

        let metrics = self.get_metrics(id, scale);

        let glyph_lines = glyph.build_lines::<true>(self.head.units_per_em as f32, scale);

        let width = (scale * glyph_lines.bounds.width).ceil() as usize;
        let height = (scale * glyph_lines.bounds.height).ceil() as usize;
        let baseline = -(scale * glyph.y_max) as isize;

        let metrics = Metrics {
            width,
            height,
            advance_width: metrics.0,
            left_side_bearing: metrics.1,
            base_line: baseline,
        };

        (metrics, glyph_lines.v_lines, glyph_lines.m_lines, glyph_lines.lines)

    }
}
