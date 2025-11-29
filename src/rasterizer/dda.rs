#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::geometry::lines::Line;
use crate::vec;
use crate::Vec;

pub struct Rasterizer {
    width: usize,
    height: usize,
    pub(crate) coverage_buffer: Vec<f32>,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, coverage_buffer: vec![0.0; width * height + 1] }
    }

    pub fn with_capacity(width: usize, height: usize) -> Self {
         Self { width, height, coverage_buffer: Vec::with_capacity(width * height + 1) }
    }

    pub fn reset(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        let len = width * height + 1;
        if self.coverage_buffer.len() < len {
            self.coverage_buffer.resize(len, 0.0);
        }
        self.coverage_buffer[..len].fill(0.0);
    }

    pub fn draw(&mut self, v_lines: &[Line], m_lines: &[Line]) -> &mut Self {
        for line in v_lines {
            self.v_line(line);
        }

        for line in m_lines {
            self.m_line(line);
        }

        self
    }

    fn v_line(&mut self, line: &Line) {
        let x = line.x0 as i32;
        // Optimization: Vertical lines have constant X. If out of bounds, skip entirely.
        if x < 0 || x >= self.width as i32 {
            return;
        }

        let mut y = line.y0 as i32;
        let y_end = line.y1 as i32;

        let mut y_cross = if line.dy_sign > 0 { y as f32 + 1.0 } else { y as f32 };
        let mut y_prev = line.y0;

        let mid_x = (line.x0 - x as f32).clamp(0.0, 1.0);

        loop {
            // X check removed as it's checked above
            if y >= 0 && y < self.height as i32 {
                let idx = (x + y * self.width as i32) as usize;
                unsafe { self.add_coverage(idx, (y_prev - y_cross).clamp(-1.0, 1.0), mid_x); }
                y_prev = y_cross;
            }

            if y == y_end {
                break;
            }

            y += line.dy_sign;
            y_cross += line.dy_sign as f32;
        }

        if y >= 0 && y < self.height as i32 {
            let idx = (x + y * self.width as i32) as usize;
            unsafe { self.add_coverage(idx, (y_prev - line.y1).clamp(-1.0, 1.0), mid_x); }
        }
    }

    fn m_line(&mut self, line: &Line) {
        let x0 = line.x0;
        let y0 = line.y0;
        let x1 = line.x1;
        let y1 = line.y1;

        let dy = line.dy;
        let dx = line.dx;

        let dt_dx = line.dt_dx;
        let dt_dy = line.dt_dy;

        let mut x = x0 as i32;
        let mut y = y0 as i32;
        let x_end = x1 as i32;
        let y_end = y1 as i32;

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
                    let mid_x = (((x_prev + x1) * 0.5) - x as f32).clamp(0.0, 1.0);
                    unsafe { self.add_coverage(idx, (y_prev - y1).clamp(-1.0, 1.0), mid_x); }
                    break;
                }

                let mut t = if t_max_x < t_max_y { t_max_x } else { t_max_y };
                let mut is_clip = false;
                if t >= 1.0 {
                    t = 1.0;
                    is_clip = true;
                }

                let x_next = x0 + t * dx;
                let y_next = y0 + t * dy;

                let mid_x = (((x_prev + x_next) * 0.5) - x as f32).clamp(0.0, 1.0);
                unsafe { self.add_coverage(idx, (y_prev - y_next).clamp(-1.0, 1.0), mid_x); }

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
    unsafe fn add_coverage(&mut self, idx: usize, height: f32, mid_x: f32) {
        let m = height * mid_x;
        let left = height - m;
        let right = m;

        unsafe {
            *self.coverage_buffer.get_unchecked_mut(idx) += left;
            *self.coverage_buffer.get_unchecked_mut(idx + 1) += right;
        }
    }


    pub fn to_bitmap(&self) -> Vec<u8> {
        let len = self.width * self.height;
        let mut out = vec![0u8; len];
        
        // Use SIMD optimized accumulation and mapping
        // coverage_buffer is larger than len, so slice it.
        crate::rasterizer::simd::accumulate_and_map(&self.coverage_buffer[..len], &mut out);
        out
    }
}