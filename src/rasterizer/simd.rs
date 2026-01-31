#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

pub fn accumulate_and_map(input: &[i32], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());

    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(feature = "std")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { accumulate_and_map_avx2(input, output); return; }
            }
        }
        #[cfg(not(feature = "std"))]
        {
            #[cfg(target_feature = "avx2")]
            { unsafe { accumulate_and_map_avx2(input, output); return; } }
        }
        unsafe { accumulate_and_map_sse2(input, output); return; }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe { accumulate_and_map_neon(input, output); return; }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    accumulate_and_map_scalar(input, output);
}

#[inline(always)]
fn apply_strong_core(val_fixed: i32) -> u8 {
    let x = val_fixed.abs() as f32 / 1024.0;
    let x = if x > 1.0 { 1.0 } else { x };
    // Quadratic approximation of c^0.75: x * (2 - x)
    // This pushes 0.5 to 0.75 and 0.8 to 0.96.
    let gamma = x * (2.0 - x);
    (gamma * 255.0) as u8
}

fn accumulate_and_map_scalar(input: &[i32], output: &mut [u8]) {
    let mut acc = 0i32;
    for (i, val) in input.iter().enumerate() {
        acc += val;
        output[i] = apply_strong_core(acc);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn accumulate_and_map_sse2(input: &[i32], output: &mut [u8]) {
    let len = input.len();
    let mut i = 0;
    let mut running_acc = _mm_setzero_si128();
    let v_1024_f = _mm_set1_ps(1024.0);
    let v_255 = _mm_set1_ps(255.0);
    let v_2_0 = _mm_set1_ps(2.0);
    let v_1_0 = _mm_set1_ps(1.0);
    let v_abs_mask = _mm_castsi128_ps(_mm_set1_epi32(0x7FFFFFFF));

    while i + 4 <= len {
        let delta = _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i);
        let s1 = _mm_slli_si128(delta, 4);
        let x = _mm_add_epi32(delta, s1);
        let s2 = _mm_slli_si128(x, 8);
        let prefix = _mm_add_epi32(x, s2);
        let current_accs = _mm_add_epi32(prefix, running_acc);
        running_acc = _mm_shuffle_epi32(current_accs, 0xFF); 

        // SIMD Strong Core Math
        let accs_f = _mm_cvtepi32_ps(current_accs);
        let x = _mm_div_ps(_mm_and_ps(accs_f, v_abs_mask), v_1024_f);
        let x_clamped = _mm_min_ps(x, v_1_0);
        
        // gamma = x * (2 - x)
        let gamma = _mm_mul_ps(x_clamped, _mm_sub_ps(v_2_0, x_clamped));
        let scaled = _mm_mul_ps(gamma, v_255);
        let pixels_i32 = _mm_cvtps_epi32(scaled);

        let val_i16 = _mm_packs_epi32(pixels_i32, _mm_setzero_si128());
        let val_u8 = _mm_packus_epi16(val_i16, _mm_setzero_si128());
        let pixel_val = _mm_cvtsi128_si32(val_u8);
        core::ptr::copy_nonoverlapping(&pixel_val as *const i32 as *const u8, output.as_mut_ptr().add(i), 4);

        i += 4;
    }

    let mut current_scalar_acc = _mm_cvtsi128_si32(running_acc);
    while i < len {
        current_scalar_acc += *input.get_unchecked(i);
        output[i] = apply_strong_core(current_scalar_acc);
        i += 1;
    }
}

#[cfg(target_arch = "x86_64")]
#[cfg(any(feature = "std", target_feature = "avx2"))]
#[target_feature(enable = "avx2")]
unsafe fn accumulate_and_map_avx2(input: &[i32], output: &mut [u8]) {
    let len = input.len();
    let mut i = 0;
    let mut running_acc_val = 0i32;
    let v_1024_f = _mm256_set1_ps(1024.0);
    let v_255 = _mm256_set1_ps(255.0);
    let v_2_0 = _mm256_set1_ps(2.0);
    let v_1_0 = _mm256_set1_ps(1.0);
    let v_abs_mask = _mm256_castsi256_ps(_mm256_set1_epi32(0x7FFFFFFF));

    while i + 8 <= len {
        let delta = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);
        let s1 = _mm256_bslli_epi128(delta, 4);
        let x = _mm256_add_epi32(delta, s1);
        let s2 = _mm256_bslli_epi128(x, 8);
        let prefix = _mm256_add_epi32(x, s2);
        let last_low = _mm256_permute2x128_si256(prefix, prefix, 0x00);
        let last_low = _mm256_shuffle_epi32(last_low, 0xFF);
        let last_low = _mm256_blend_epi32(_mm256_setzero_si256(), last_low, 0xF0);
        let prefix = _mm256_add_epi32(prefix, last_low);

        let current_accs = _mm256_add_epi32(prefix, _mm256_set1_epi32(running_acc_val));
        
        // Extract last element for next running_acc
        let mut last_accs = [0i32; 8];
        _mm256_storeu_si256(last_accs.as_mut_ptr() as *mut __m256i, current_accs);
        running_acc_val = last_accs[7];

        let accs_f = _mm256_cvtepi32_ps(current_accs);
        let x = _mm256_div_ps(_mm256_and_ps(accs_f, v_abs_mask), v_1024_f);
        let x_clamped = _mm256_min_ps(x, v_1_0);
        let gamma = _mm256_mul_ps(x_clamped, _mm256_sub_ps(v_2_0, x_clamped));
        let scaled = _mm256_mul_ps(gamma, v_255);
        let pixels_i32 = _mm256_cvtps_epi32(scaled);

        // Pack 8x i32 -> 8x u8
        let low_128 = _mm256_castsi256_si128(pixels_i32);
        let high_128 = _mm256_extracti128_si256(pixels_i32, 1);
        let packed_16 = _mm_packs_epi32(low_128, high_128);
        let packed_8 = _mm_packus_epi16(packed_16, packed_16);
        
        _mm_storel_epi64(output.as_mut_ptr().add(i) as *mut __m128i, packed_8);

        i += 8;
    }

    while i < len {
        running_acc_val += *input.get_unchecked(i);
        output[i] = apply_strong_core(running_acc_val);
        i += 1;
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn accumulate_and_map_neon(input: &[i32], output: &mut [u8]) {
    let len = input.len();
    let mut i = 0;
    let mut running_acc_val = 0i32;
    let v_1024_f = vdupq_n_f32(1024.0);
    let v_255 = vdupq_n_f32(255.0);
    let v_2_0 = vdupq_n_f32(2.0);
    let v_1_0 = vdupq_n_f32(1.0);

    while i + 4 <= len {
        let delta = vld1q_s32(input.as_ptr().add(i));
        let zero = vdupq_n_s32(0);
        let s1 = vextq_s32(zero, delta, 3); 
        let x = vaddq_s32(delta, s1);
        let s2 = vextq_s32(zero, x, 2);
        let prefix = vaddq_s32(x, s2);
        let current_accs = vaddq_s32(prefix, vdupq_n_s32(running_acc_val));
        running_acc_val = vgetq_lane_s32(current_accs, 3);

        let accs_f = vcvtq_f32_s32(current_accs);
        let x = vdivq_f32(vabsq_f32(accs_f), v_1024_f);
        let x_clamped = vminq_f32(x, v_1_0);
        let gamma = vmulq_f32(x_clamped, vsubq_f32(v_2_0, x_clamped));
        let scaled = vmulq_f32(gamma, v_255);
        let pixels_u32 = vcvtq_u32_f32(scaled);

        let packed_16 = vqmovn_u32(pixels_u32);
        let packed_8 = vqmovn_u16(vcombine_u16(packed_16, packed_16));
        
        vst1_lane_u32(output.as_mut_ptr().add(i) as *mut u32, vreinterpret_u32_u8(packed_8), 0);

        i += 4;
    }

    while i < len {
        running_acc_val += *input.get_unchecked(i);
        output[i] = apply_strong_core(running_acc_val);
        i += 1;
    }
}