use crate::Vec;
use crate::geometry::lines::{QMODE_LINEAR, QMODE_QUAD};
use crate::geometry::simd::f32x4;

const SLACK: usize = 4;

/// Single-instruction f32 sqrt on x86; Newton refinement from a bit-hack
/// seed elsewhere (core has no f32::sqrt in no_std).
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
fn sqrt_f(v: f32) -> f32 {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{_mm_cvtss_f32, _mm_set_ss, _mm_sqrt_ss};
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{_mm_cvtss_f32, _mm_set_ss, _mm_sqrt_ss};
    unsafe { _mm_cvtss_f32(_mm_sqrt_ss(_mm_set_ss(v))) }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
#[inline(always)]
fn sqrt_f(v: f32) -> f32 {
    if v <= 0.0 {
        return 0.0;
    }
    let mut g = f32::from_bits((v.to_bits() >> 1) + 0x1FC0_0000);
    g = 0.5 * (g + v / g);
    g = 0.5 * (g + v / g);
    g = 0.5 * (g + v / g);
    g
}

/// Parameter of the next boundary crossing for one axis of a monotonic
/// quadratic piece. `cmb` is kept exact (start - boundary) so the
/// discriminant is recomputed drift-free at every crossing.
#[inline(always)]
fn next_t(mode: u8, b2: f32, four_a: f32, cmb: f32, b: f32, inv: f32, dir: f32) -> f32 {
    if mode == QMODE_QUAD {
        (dir * sqrt_f((b2 - four_a * cmb).max(0.0)) - b) * inv
    } else if mode == QMODE_LINEAR {
        -cmb * inv
    } else {
        f32::INFINITY
    }
}

/// Truncating f32 -> i32 without Rust's saturation fixup code. `cvttss2si`
/// is a single instruction; the values fed here are clamped glyph
/// coordinates, far inside i32 range.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
fn trunc_i32(v: f32) -> i32 {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{_mm_cvttss_si32, _mm_set_ss};
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{_mm_cvttss_si32, _mm_set_ss};
    unsafe { _mm_cvttss_si32(_mm_set_ss(v)) }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
#[inline(always)]
fn trunc_i32(v: f32) -> i32 {
    v as i32
}

/// Fractional part for non-negative values (all raster coords are clamped
/// to [0, dim)): one convert down, one convert up, one subtract.
#[inline(always)]
fn fract_pos(v: f32) -> f32 {
    v - trunc_i32(v) as f32
}

pub struct Rasterizer {
    width: usize,
    height: usize,
    pub(crate) coverage_buffer: Vec<f32>,
    pub(crate) scanline_bounds: Vec<(u16, u16)>,
    used_len: usize,
    /// True when `to_bitmap` has re-zeroed every span the last draw touched,
    /// letting `reset` skip the coverage clear entirely.
    clean: bool,
}

impl Rasterizer {
    pub fn with_capacity(width: usize, height: usize) -> Self {
        let len = width * height + SLACK;
        Self {
            width,
            height,
            coverage_buffer: Vec::with_capacity(len),
            scanline_bounds: Vec::with_capacity(height),
            used_len: 0,
            clean: true,
        }
    }

    pub fn reset(&mut self, width: usize, height: usize) {
        // Clear what the previous glyph dirtied. The coverage buffer is
        // usually already clean: to_bitmap zeroes every touched span as it
        // accumulates, so only an aborted draw pays for a full clear.
        if !self.clean {
            let stale = self.used_len.min(self.coverage_buffer.len());
            self.coverage_buffer[..stale].fill(0.0);
        }
        let stale_rows = self.height.min(self.scanline_bounds.len());
        self.scanline_bounds[..stale_rows].fill((u16::MAX, 0));

        self.width = width;
        self.height = height;
        if width == 0 || height == 0 {
            self.used_len = 0;
            self.clean = true;
            return;
        }

        let len = width * height + SLACK;
        if self.coverage_buffer.len() < len {
            self.coverage_buffer.resize(len, 0.0);
        }
        if self.scanline_bounds.len() < height {
            self.scanline_bounds.resize(height, (u16::MAX, 0));
        }

        self.used_len = len;
        self.clean = false;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        v_lines: &[f32x4],
        m_lines: &[f32x4],
        m_params: &[f32x4],
        quad_q0: &[f32x4],
        quad_q1: &[f32x4],
        quad_q2: &[f32x4],
        quad_modes: &[u8],
        scale: f32,
        shift_x: f32,
        shift_y: f32,
        width_px: f32,
        height_px: f32,
    ) -> &mut Self {
        if self.width == 0 || self.height == 0 {
            return self;
        }

        // Font units -> buffer space (y flipped) folded into a single
        // multiply-add per line: x' = x*s - shift_x*s,
        // y' = -y*s + (shift_y*s + H).
        let mul = f32x4::new(scale, -scale, scale, -scale);
        let add = f32x4::new(
            -shift_x * scale,
            shift_y * scale + height_px,
            -shift_x * scale,
            shift_y * scale + height_px,
        );
        // Clamp epsilon keeps trunc() strictly below the dimensions so cell
        // indices never touch width/height.
        let eps = 0.0001;
        let min_vec = f32x4::zero();
        let max_vec = f32x4::new(width_px - eps, height_px - eps, width_px - eps, height_px - eps);
        // Params rescale: (1/|dx|, 1/|dy|, dx, dy) in font units ->
        // (1/(|dx|s), 1/(|dy|s), dx*s, -dy*s) in buffer space (y flipped).
        let inv_s = 1.0 / scale;
        let pmul = f32x4::new(inv_s, inv_s, scale, -scale);
        // Quad coefficient rescales. Coefficients are difference vectors:
        // no offset, y components negated by the flip. Reciprocals divide
        // by the scale (and flip sign in y).
        let qc_mul = mul; // (s, -s, s, -s) for (a_x, a_y, b_x, b_y)
        let qi_mul = f32x4::new(inv_s, -inv_s, 0.0, 0.0);

        for &line_vec in v_lines {
            let scaled = (line_vec * mul + add).clamp(min_vec, max_vec);
            let (x0, y0, _, y1) = scaled.copied();
            if y0 != y1 {
                self.v_line(x0, y0, y1);
            }
        }

        for i in 0..m_lines.len() {
            let scaled = (m_lines[i] * mul + add).clamp(min_vec, max_vec);
            let (x0, y0, x1, y1) = scaled.copied();
            if y0 != y1 {
                let (tdx, tdy, dx, dy) = (m_params[i] * pmul).copied();
                self.m_line(x0, y0, x1, y1, dx, dy, tdx, tdy);
            }
        }

        for i in 0..quad_q0.len() {
            // Monotonic piece: the curve interior stays inside the endpoint
            // box, so clamping the endpoints bounds the whole walk.
            let ends = (quad_q0[i] * mul + add).clamp(min_vec, max_vec);
            let (x0, y0, x1, y1) = ends.copied();
            if y0 != y1 {
                let (ax, ay, bx, by) = (quad_q1[i] * qc_mul).copied();
                let (ivx, ivy, _, _) = (quad_q2[i] * qi_mul).copied();
                self.q_curve(x0, y0, x1, y1, ax, ay, bx, by, ivx, ivy, quad_modes[i]);
            }
        }

        self
    }

    #[inline(always)]
    fn add(&mut self, idx: usize, x: u16, y: usize, height: f32, mid_x: f32) {
        debug_assert!(idx < self.coverage_buffer.len());
        debug_assert!(y < self.scanline_bounds.len());
        unsafe {
            let m = height * mid_x;
            *self.coverage_buffer.get_unchecked_mut(idx) += height - m;

            let bounds = self.scanline_bounds.get_unchecked_mut(y);
            bounds.0 = bounds.0.min(x);

            if (x as usize) + 1 == self.width {
                // Right-edge cell: the spill fraction has nowhere to go
                // within the row. Dropping it is harmless because the
                // accumulate pass restarts at zero on every row.
                bounds.1 = bounds.1.max(x + 1);
            } else {
                *self.coverage_buffer.get_unchecked_mut(idx + 1) += m;
                bounds.1 = bounds.1.max(x + 2);
            }
        }
    }

    fn v_line(&mut self, x0: f32, y0: f32, y1: f32) {
        let xi = trunc_i32(x0);
        let y0i = trunc_i32(y0);
        let y1i = trunc_i32(y1);
        let mid_x = x0 - xi as f32;

        let down = y1 > y0;
        let sy = if down { 1.0 } else { -1.0 };
        let mut target_y = if down { (y0i + 1) as f32 } else { y0i as f32 };
        let mut y_prev = y0;

        let index_inc: i32 = if down { self.width as i32 } else { -(self.width as i32) };
        let cy_inc: i32 = if down { 1 } else { -1 };
        let mut dist = (y1i - y0i).abs();

        let cx = xi as u16;
        let mut cy = y0i;
        let mut index = xi + y0i * self.width as i32;

        while dist > 0 {
            dist -= 1;
            self.add(index as usize, cx, cy as usize, y_prev - target_y, mid_x);
            index += index_inc;
            cy += cy_inc;
            y_prev = target_y;
            target_y += sy;
        }
        self.add(index as usize, cx, cy as usize, y_prev - y1, mid_x);
    }

    #[allow(clippy::too_many_arguments)]
    fn m_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, dx: f32, dy: f32, tdx: f32, tdy: f32) {
        let x0i = trunc_i32(x0);
        let y0i = trunc_i32(y0);
        let x1i = trunc_i32(x1);
        let y1i = trunc_i32(y1);

        let right = dx > 0.0;
        let down = dy > 0.0;
        let sx = if right { 1.0 } else { -1.0 };
        let sy = if down { 1.0 } else { -1.0 };

        let mut target_x = if right { (x0i + 1) as f32 } else { x0i as f32 };
        let mut target_y = if down { (y0i + 1) as f32 } else { y0i as f32 };

        let mut tmx = (target_x - x0).abs() * tdx;
        let mut tmy = (target_y - y0).abs() * tdy;

        let mut x_prev = x0;
        let mut y_prev = y0;

        let cx_inc: i32 = if right { 1 } else { -1 };
        let cy_inc: i32 = if down { 1 } else { -1 };
        let mut cx = x0i;
        let mut cy = y0i;

        let mut index = x0i + y0i * self.width as i32;
        let index_x_inc: i32 = cx_inc;
        let index_y_inc: i32 = if down { self.width as i32 } else { -(self.width as i32) };

        let mut dist_x = (x1i - x0i).abs();
        let mut dist_y = (y1i - y0i).abs();
        let mut dist = dist_x + dist_y;

        while dist > 0 {
            dist -= 1;
            let prev_index = index;
            let prev_x = cx as u16;
            let prev_y = cy as usize;
            let y_next: f32;
            let x_next: f32;
            // The integer dist counters keep the walk in lockstep with the
            // cell indices even when float noise in tmx/tmy disagrees.
            if dist_x > 0 && (dist_y == 0 || tmx < tmy) {
                y_next = tmx * dy + y0;
                x_next = target_x;
                tmx += tdx;
                target_x += sx;
                index += index_x_inc;
                cx += cx_inc;
                dist_x -= 1;
            } else {
                y_next = target_y;
                x_next = tmy * dx + x0;
                tmy += tdy;
                target_y += sy;
                index += index_y_inc;
                cy += cy_inc;
                dist_y -= 1;
            }
            self.add(prev_index as usize, prev_x, prev_y, y_prev - y_next, fract_pos((x_prev + x_next) * 0.5));
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add(index as usize, cx as u16, cy as usize, y_prev - y1, fract_pos((x_prev + x1) * 0.5));
    }

    /// Walk one monotonic quadratic piece exactly like `m_line`, but with
    /// curve intersections solved via the quadratic formula per crossing.
    /// Curves are therefore rendered without any flattening.
    #[allow(clippy::too_many_arguments)]
    fn q_curve(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        ax: f32,
        ay: f32,
        bx: f32,
        by: f32,
        ivx: f32,
        ivy: f32,
        mode: u8,
    ) {
        let mode_x = mode & 0x0F;
        let mode_y = mode >> 4;

        let x0i = trunc_i32(x0);
        let y0i = trunc_i32(y0);
        let x1i = trunc_i32(x1);
        let y1i = trunc_i32(y1);

        let sx: i32 = if x1 > x0 { 1 } else { -1 };
        let sy: i32 = if y1 > y0 { 1 } else { -1 };
        let sxf = sx as f32;
        let syf = sy as f32;

        // Next pixel boundary in each axis, and (start - boundary), which
        // stays exact as the boundary advances by whole pixels.
        let mut bxv = if sx > 0 { (x0i + 1) as f32 } else { x0i as f32 };
        let mut byv = if sy > 0 { (y0i + 1) as f32 } else { y0i as f32 };
        let mut cmbx = x0 - bxv;
        let mut cmby = y0 - byv;

        let four_ax = 4.0 * ax;
        let four_ay = 4.0 * ay;
        let b2x = bx * bx;
        let b2y = by * by;

        let mut tx = next_t(mode_x, b2x, four_ax, cmbx, bx, ivx, sxf);
        let mut ty = next_t(mode_y, b2y, four_ay, cmby, by, ivy, syf);

        // Monotonicity bounds the curve inside the endpoint box; clamping
        // evaluated coordinates guards the deposit math against float noise.
        let (xlo, xhi) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
        let (ylo, yhi) = if y0 < y1 { (y0, y1) } else { (y1, y0) };

        let mut x_prev = x0;
        let mut y_prev = y0;
        let mut cx = x0i;
        let mut cy = y0i;
        let mut index = x0i + y0i * self.width as i32;
        let index_y_inc = if sy > 0 { self.width as i32 } else { -(self.width as i32) };

        let mut dist_x = (x1i - x0i).abs();
        let mut dist_y = (y1i - y0i).abs();
        let mut dist = dist_x + dist_y;

        while dist > 0 {
            dist -= 1;
            let prev_index = index;
            let px = cx as u16;
            let py = cy as usize;
            let x_next: f32;
            let y_next: f32;
            if dist_x > 0 && (dist_y == 0 || tx < ty) {
                x_next = bxv;
                y_next = ((ay * tx + by) * tx + y0).clamp(ylo, yhi);
                bxv += sxf;
                cmbx -= sxf;
                tx = next_t(mode_x, b2x, four_ax, cmbx, bx, ivx, sxf);
                index += sx;
                cx += sx;
                dist_x -= 1;
            } else {
                y_next = byv;
                x_next = ((ax * ty + bx) * ty + x0).clamp(xlo, xhi);
                byv += syf;
                cmby -= syf;
                ty = next_t(mode_y, b2y, four_ay, cmby, by, ivy, syf);
                index += index_y_inc;
                cy += sy;
                dist_y -= 1;
            }
            self.add(
                prev_index as usize,
                px,
                py,
                y_prev - y_next,
                fract_pos((x_prev + x_next) * 0.5),
            );
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add(
            index as usize,
            cx as u16,
            cy as usize,
            y_prev - y1,
            fract_pos((x_prev + x1) * 0.5),
        );
    }

    // `&mut self` is deliberate despite the `to_` name: accumulation
    // re-zeroes the coverage buffer as it converts.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bitmap(&mut self) -> Vec<u8> {
        let len = self.width * self.height;
        // Skip zero-init: accumulate_and_map writes every byte.
        let mut out = Vec::with_capacity(len);
        #[allow(clippy::uninit_vec)]
        unsafe {
            out.set_len(len);
        }

        crate::geometry::simd::accumulate_and_map(
            &mut self.coverage_buffer[..len],
            &self.scanline_bounds[..self.height],
            self.width,
            self.height,
            &mut out,
        );
        // accumulate_and_map re-zeroed every span it read; the buffer is
        // clean again and reset() can skip its clear.
        self.clean = true;
        out
    }
}
