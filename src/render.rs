use crate::font::TrueTypeFont;
use crate::F32NoStd;
use crate::rasterizer::aet::rasterize;
use crate::rasterizer::point::Contour;
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
    pub fn get_char<const CACHE: bool>(&mut self, c: char, size: usize) -> (Metrics, Vec<u8>) {

        let scale = size as f32 / self.head.units_per_em as f32;

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

        let width = (((glyph.x_max - glyph.x_min) as f32 * scale).ceil() as usize);
        let height = (((glyph.y_max - glyph.y_min) as f32 * scale).ceil() as usize) + 1;
        let baseline = -(glyph.y_max as f32 * scale) as isize;

        let required_size = width * height;

        let extra = self.get_metrics(id, scale);
        let metrics = Metrics {
            width,
            height,
            advance_width: extra.0,
            left_side_bearing: extra.1,
            base_line: baseline,
        };

        self.bitmap_buffer.resize(required_size, 0);
        self.bitmap_buffer[..required_size].fill(0);

        rasterize(&glyph.points, scale, glyph.y_max as f32, glyph.x_min as f32, width, height, &mut self.bitmap_buffer);


        if CACHE {
            self.cache.set(*id, size, metrics.clone(), self.bitmap_buffer.clone());
        }

        (metrics, self.bitmap_buffer.clone())
    }
}

fn show_points(points: &[Contour], scale: f32, y_max: f32, x_min: f32, width: usize, height: usize, bitmap_buffer: &mut Vec<u8>) {
    for contour in points {
        for p in &contour.points {
            if p.on_curve {
                let x = ((p.x as f32 - x_min) * scale).round() as isize;
                let y = ((y_max - p.y as f32) * scale).round() as isize;

                let idx = y as usize * width + x as usize;
                bitmap_buffer[idx] = 255;
            } else {
                let x = ((p.x as f32 - x_min) * scale).round() as isize;
                let y = ((y_max - p.y as f32) * scale).round() as isize;

                let idx = y as usize * width + x as usize;
                bitmap_buffer[idx] = 150;
            }
        }
    }
}

