#![allow(non_camel_case_types)]

use core::ops::{Add, Div, Mul, Sub};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::arch::x86_64::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct f32x4(
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub(crate) __m128,
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub(crate) [f32; 4]
);

impl f32x4 {
    #[inline(always)]
    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32) -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            f32x4(unsafe { _mm_set_ps(x3, x2, x1, x0) })
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([x0, x1, x2, x3])
        }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            f32x4(unsafe { _mm_setzero_ps() })
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([0.0, 0.0, 0.0, 0.0])
        }
    }

    #[inline(always)]
    pub fn copied(self) -> (f32, f32, f32, f32) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { core::mem::transmute::<__m128, (f32, f32, f32, f32)>(self.0) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            (self.0[0], self.0[1], self.0[2], self.0[3])
        }
    }

    #[inline(always)]
    pub fn min(self, other: f32x4) -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_min_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0].min(other.0[0]),
                self.0[1].min(other.0[1]),
                self.0[2].min(other.0[2]),
                self.0[3].min(other.0[3]),
            ])
        }
    }

    #[inline(always)]
    pub fn max(self, other: f32x4) -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_max_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0].max(other.0[0]),
                self.0[1].max(other.0[1]),
                self.0[2].max(other.0[2]),
                self.0[3].max(other.0[3]),
            ])
        }
    }

    #[inline(always)]
    pub fn clamp(self, min: f32x4, max: f32x4) -> Self {
        self.max(min).min(max)
    }
}

impl Add for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn add(self, other: f32x4) -> f32x4 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_add_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0] + other.0[0],
                self.0[1] + other.0[1],
                self.0[2] + other.0[2],
                self.0[3] + other.0[3],
            ])
        }
    }
}

impl Sub for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn sub(self, other: f32x4) -> f32x4 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_sub_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0] - other.0[0],
                self.0[1] - other.0[1],
                self.0[2] - other.0[2],
                self.0[3] - other.0[3],
            ])
        }
    }
}

/// Accumulates float winding coverage into bytes for each row's
/// touched span, and re-zeroes the coverage it reads so the buffer comes out
/// clean. Row-local accumulation prevents cross-row drift.
#[inline(never)]
pub fn accumulate_and_map(
    a: &mut [f32],
    bounds: &[(u16, u16)],
    width: usize,
    height: usize,
    out: &mut [u8],
) {
    assert_eq!(a.len(), width * height);
    assert_eq!(out.len(), width * height);
    assert_eq!(bounds.len(), height);

    for (y, &(min_x, max_x)) in bounds.iter().enumerate() {
        let offset = y * width;

        let out_row = unsafe { out.get_unchecked_mut(offset..offset + width) };

        if min_x > max_x {
            out_row.fill(0);
            continue;
        }

        let min_x = (min_x as usize).min(width);
        let max_x = (max_x as usize).min(width);
        let a_row = unsafe { a.get_unchecked_mut(offset..offset + width) };

        out_row[..min_x].fill(0);

        let running_sum = accumulate_span(a_row, out_row, min_x, max_x);

        if max_x < width {
            let byte = clamp_alpha(running_sum);
            out_row[max_x..].fill(byte);
        }
    }
}

#[inline(always)]
fn clamp_alpha(sum: f32) -> u8 {
    (sum.abs() * 255.9) as u8
}

/// Prefix-sum `a[min_x..max_x]` into alpha bytes, zeroing the coverage as it
/// is consumed. Returns the running sum at the end of the span.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
fn accumulate_span(a: &mut [f32], out: &mut [u8], min_x: usize, max_x: usize) -> f32 {
    unsafe {
        let mut i = min_x;
        let mut offset = _mm_setzero_ps();
        let nzero = _mm_set1_ps(-0.0);
        let scale = _mm_set1_ps(255.9);
        let zero = _mm_setzero_ps();

        while i + 4 <= max_x {
            let p = a.as_mut_ptr().add(i);
            let mut x = _mm_loadu_ps(p);
            _mm_storeu_ps(p, zero);
            // 4-wide inclusive prefix sum via two shift-adds.
            x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 4)));
            x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 8)));
            x = _mm_add_ps(x, offset);

            // |x| * 255.9, truncate, saturating-pack to 4 bytes.
            let y = _mm_andnot_ps(nzero, _mm_mul_ps(x, scale));
            let y = _mm_cvttps_epi32(y);
            let y = _mm_packus_epi16(_mm_packs_epi32(y, _mm_setzero_si128()), _mm_setzero_si128());
            let px = _mm_cvtsi128_si32(y);
            core::ptr::copy_nonoverlapping(
                &px as *const i32 as *const u8,
                out.as_mut_ptr().add(i),
                4,
            );

            offset = _mm_shuffle_ps(x, x, 0xFF);
            i += 4;
        }

        let mut running = _mm_cvtss_f32(offset);
        while i < max_x {
            let v = a.get_unchecked_mut(i);
            running += *v;
            *v = 0.0;
            *out.get_unchecked_mut(i) = clamp_alpha(running);
            i += 1;
        }
        running
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline(always)]
fn accumulate_span(a: &mut [f32], out: &mut [u8], min_x: usize, max_x: usize) -> f32 {
    let mut running = 0.0f32;
    for i in min_x..max_x {
        unsafe {
            let v = a.get_unchecked_mut(i);
            running += *v;
            *v = 0.0;
            *out.get_unchecked_mut(i) = clamp_alpha(running);
        }
    }
    running
}

impl Mul for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn mul(self, other: f32x4) -> f32x4 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_mul_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0] * other.0[0],
                self.0[1] * other.0[1],
                self.0[2] * other.0[2],
                self.0[3] * other.0[3],
            ])
        }
    }
}

impl Div for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn div(self, other: f32x4) -> f32x4 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            unsafe { f32x4(_mm_div_ps(self.0, other.0)) }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            f32x4([
                self.0[0] / other.0[0],
                self.0[1] / other.0[1],
                self.0[2] / other.0[2],
                self.0[3] / other.0[3],
            ])
        }
    }
}

impl core::fmt::Debug for f32x4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (x, y, z, w) = self.copied();
        write!(f, "f32x4({}, {}, {}, {})", x, y, z, w)
    }
}
