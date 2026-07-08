use crate::Vec;
use crate::font::TrueTypeFont;

#[allow(unused_imports)]
use crate::F32NoStd;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub width: usize,
    pub height: usize,
    pub left_side_bearing: isize,
    pub advance_width: usize,
    pub base_line: isize,
}

impl TrueTypeFont {
    /// Glyph index for a character; 0 (the missing glyph) when unmapped.
    #[inline]
    pub fn lookup_glyph_index(&self, c: char) -> u32 {
        self.glyph_id_table.get(c as u64).copied().unwrap_or(0)
    }

    pub fn get_char<const CACHE: bool>(&mut self, c: char, size: f32) -> (Metrics, Vec<u8>) {
        self.get_indexed::<CACHE>(self.lookup_glyph_index(c), size)
    }

    /// Rasterize by glyph index (see [`Self::lookup_glyph_index`]). Useful
    /// when glyph ids are already resolved, e.g. by an external shaper.
    pub fn get_indexed<const CACHE: bool>(&mut self, id: u32, size: f32) -> (Metrics, Vec<u8>) {
        let scale = size * (self.dpi / 72.0) / self.head.units_per_em as f32;

        if CACHE {
            let is_cached = self.cache.get(id, size);
            if let Some(cached) = is_cached {
                return cached.clone();
            }
        }

        // Unmapped characters fall back to the missing glyph (index 0),
        // which load_font guarantees is present; if it somehow isn't,
        // render nothing rather than panic.
        let Some(glyph) = self
            .glyph_data_table
            .get(id as usize)
            .and_then(|opt| opt.as_ref())
            .or_else(|| self.glyph_data_table.first().and_then(|opt| opt.as_ref()))
        else {
            return (
                Metrics {
                    width: 0,
                    height: 0,
                    left_side_bearing: 0,
                    advance_width: 0,
                    base_line: 0,
                },
                Vec::new(),
            );
        };

        let shift_x = (glyph.min_x * scale).floor() / scale;
        let shift_y = (glyph.min_y * scale).floor() / scale;

        let width_aligned = ((glyph.max_x * scale).ceil() - (glyph.min_x * scale).floor()) / scale;
        let height_aligned = ((glyph.max_y * scale).ceil() - (glyph.min_y * scale).floor()) / scale;

        let width_scaled = width_aligned * scale;
        let height_scaled = height_aligned * scale;

        let width = width_scaled.ceil() as usize;
        let height = height_scaled.ceil() as usize;
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

        // Single path at every size: straight edges from the baked line
        // lists, curves rasterized directly as monotonic quadratics - no
        // flattening, mathematically exact at any scale.
        self.rasterizer.draw(
            &glyph.pre_v_lines,
            &glyph.pre_m_lines,
            &glyph.pre_m_params,
            &glyph.quad_q0,
            &glyph.quad_q1,
            &glyph.quad_q2,
            &glyph.quad_modes,
            scale,
            shift_x,
            shift_y,
            width as f32,
            height as f32,
        );
        let bitmap = self.rasterizer.to_bitmap();

        if CACHE {
            self.cache.set(id, size, metrics.clone(), bitmap.clone());
        }

        (metrics, bitmap)
    }
}
