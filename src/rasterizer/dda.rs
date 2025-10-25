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

        let x = x0.floor() as i32;
        let mut y = y0.floor() as i32;
        let y_end = y1.floor() as i32;

        let mut y_cross = if line.dy_sign > 0 { y as f32 + 1.0 } else { y as f32 };
        let mut y_prev = y0;

        let mid_x = (x0 - x as f32).clamp(0.0, 1.0);

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let idx = (x + y * self.width as i32) as usize;
                self.add_coverage(idx, (y_prev - y_cross).clamp(-1.0, 1.0), mid_x);
                y_prev = y_cross;
            }

            if y == y_end {
                break;
            }

            y += line.dy_sign;
            y_cross += line.dy_sign as f32;
        }

        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (x + y * self.width as i32) as usize;
            self.add_coverage(idx, (y_prev - y1).clamp(-1.0, 1.0), mid_x);
        }
    }

    #[inline(always)]
    fn m_line(&mut self, line: &Line, scale: f32) {
        let x0 = line.x0 * scale;
        let y0 = line.y0 * scale;
        let x1 = line.x1 * scale;
        let y1 = line.y1 * scale;

        let dy = line.dy * scale;
        let dx = line.dx * scale;

        let dt_dx = if !line.dx_is_zero { (line.dt_dx) / (scale) } else { f32::MAX };
        let dt_dy = if !line.dy_is_zero { (line.dt_dy) / (scale) } else { f32::MAX };

        let mut x = x0.floor() as i32;
        let mut y = y0.floor() as i32;
        let x_end = x1.floor() as i32;
        let y_end = y1.floor() as i32;

        if line.is_degen { return; }

        let mut x_cross = if line.dx_sign > 0 { x + 1 } else { x };
        let mut y_cross = if line.dy_sign > 0 { y + 1 } else { y };

        let mut t_max_x = if !line.dx_is_zero { (x_cross as f32 - x0) / dx } else { f32::MAX };
        let mut t_max_y = if !line.dy_is_zero { (y_cross as f32 - y0) / dy } else { f32::MAX };

        let mut x_prev = x0;
        let mut y_prev = y0;

        loop {
            let at_end = x == x_end && y == y_end;

            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let idx = (y as usize) * self.width + (x as usize);

                if at_end {
                    let mid_x = (((x_prev + x1 as f32) * 0.5) - x as f32).clamp(0.0, 1.0);
                    self.add_coverage(idx, (y_prev - y1 as f32).clamp(-1.0, 1.0), mid_x);
                    break;
                }

                let mut t = if t_max_x < t_max_y { t_max_x } else { t_max_y };
                let mut is_clip = false;
                if t >= 1.0 {
                    t = 1.0;
                    is_clip = true;
                }

                let x_next = (x0 + t * dx) as f32;
                let y_next = (y0 + t * dy) as f32;

                let mid_x = (((x_prev + x_next) * 0.5) - x as f32).clamp(0.0, 1.0);
                self.add_coverage(idx, (y_prev - y_next).clamp(-1.0, 1.0), mid_x);

                x_prev = x_next;
                y_prev = y_next;

                if is_clip {
                    break;
                }
            } else if t_max_x >= 1.0 && t_max_y >= 1.0 {
                break;
            }

            if t_max_x < t_max_y {
                x += line.dx_sign;
                x_cross += line.dx_sign;
                t_max_x += dt_dx;
            } else {
                y += line.dy_sign;
                y_cross += line.dy_sign;
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
        if idx + 1 < self.coverage_buffer.len() {
            self.coverage_buffer[idx + 1] += right;
        }
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