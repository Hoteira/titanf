#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::preprocess::lines::Line;
use crate::tables::glyf::Glyph;
use crate::vec;
use crate::Vec;

pub struct Rasterizer {
    width: usize,
    height: usize,
    pub(crate) coverage_buffer: Vec<f32>,
}

impl Rasterizer {
    #[inline(always)]
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, coverage_buffer: vec![0.0; width * height + 1] }
    }

    #[inline(always)]
    pub(crate) fn draw(mut self, glyph: &Glyph, scale: f32) -> Self{

        for line in &glyph.v_lines {
            self.v_line(line, scale);
        }

        for line in &glyph.m_lines {
             self.m_line(line, scale);
        }

        self
    }

    #[inline(always)]
    fn v_line(&mut self, line: &Line, scale: f32) {

        let x0 = line.x0 * scale;
        let y0 = line.y0 * scale;
        let y1 = line.y1 * scale;
        let dy = line.dy.signum() as i32;

        let x = x0.floor() as i32;
        let mut y = y0.floor() as i32;
        let y_end = y1.floor() as i32;

        let mut y_cross = if dy > 0 { y as f32 + 1.0 } else { y as f32 };
        let mut y_prev = y0;

        let mid_x = (x0 - x as f32).clamp(0.0, 1.0);

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let idx = (x + y * self.width as i32) as usize;
                self.add_coverage(idx, (y_prev - y_cross), mid_x);
                y_prev = y_cross;
            }

            if y == y_end {
                break;
            }

            y += dy;
            y_cross += dy as f32;
        }

        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (x + y * self.width as i32) as usize;
            self.add_coverage(idx, (y_prev - y1), mid_x);
        }
    }

    #[inline(always)]
    fn m_line(&mut self,  line: &Line, scale: f32) {

        let x0 = line.x0 * scale;
        let y0 = line.y0 * scale;
        let x1 = line.x1 * scale;
        let y1 = line.y1 * scale;

        let dy = line.dy * scale;
        let dx = line.dx * scale;

        let dt_dx = if dx != 0.0 { 1.0 / dx.abs() } else { f32::MAX };
        let dt_dy = if dy != 0.0 { 1.0 / dy.abs() } else { f32::MAX };

        let dx_sign = dx.signum() as i32;
        let dy_sign = dy.signum() as i32;

        let mut x = x0.floor() as i32;
        let mut y = y0.floor() as i32;
        let x_end = x1.floor() as i32;
        let y_end = y1.floor() as i32;

        if dx == 0.0 && dy == 0.0 {
            return;
        }

        let mut x_cross = if dx_sign > 0 { x as f32 + 1.0 } else { x as f32 };
        let mut y_cross = if dy_sign > 0 { y as f32 + 1.0 } else { y as f32 };

        let mut t_max_x = if dx != 0.0 { (x_cross - x0) / dx } else { f32::MAX };
        let mut t_max_y = if dy != 0.0 { (y_cross - y0) / dy } else { f32::MAX };

        let mut x_prev = x0;
        let mut y_prev = y0;

        loop {
            let at_end = x == x_end && y == y_end;

            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let idx = (x + y * self.width as i32) as usize;

                if at_end {
                    let mid_x = (((x_prev + x1) * 0.5) - x as f32).clamp(0.0, 1.0);
                    self.add_coverage(idx, (y_prev - y1), mid_x);
                    break;
                }

                let (x_next, y_next) = if t_max_x < t_max_y {
                    let t = t_max_x;
                    (x_cross, y0 + t * dy)
                } else {
                    let t = t_max_y;
                    (x0 + t * dx, y_cross)
                };

                let mid_x = (((x_prev + x_next) * 0.5) - x as f32).clamp(0.0, 1.0);
                self.add_coverage(idx, y_prev - y_next, mid_x);

                x_prev = x_next;
                y_prev = y_next;
            }

            if at_end {
                break;
            }

            if t_max_x < t_max_y {
                x += dx_sign;
                x_cross += dx_sign as f32;
                t_max_x += dt_dx;
            } else {
                y += dy_sign;
                y_cross += dy_sign as f32;
                t_max_y += dt_dy;
            }
        }
    }

    #[inline(always)]
    fn add_coverage(&mut self, idx: usize, height: f32, mid_x: f32) {
        let m = height * mid_x;
        let left = height - m;
        let right = m;

        self.coverage_buffer[idx] += left;
        self.coverage_buffer[idx + 1] += right;
    }

    #[inline(always)]
    pub fn to_bitmap(&self) -> Vec<u8> {
        let mut out = vec![0u8; self.width * self.height];
        let mut acc = 0.0f32;
        for i in 0..self.width * self.height {
            acc += self.coverage_buffer[i];
            out[i] = (acc.abs().clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }
}