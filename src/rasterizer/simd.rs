#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

pub fn accumulate_and_map(input: &[f32], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());

    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 is guaranteed to have SSE2 in Rust's standard targets.
        // If we wanted to be pedantic we'd check, but for this task we assume x86_64 availability.
        unsafe {
            accumulate_and_map_sse2(input, output);
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            accumulate_and_map_neon(input, output);
            return;
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    accumulate_and_map_scalar(input, output);
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn accumulate_and_map_scalar(input: &[f32], output: &mut [u8]) {
    let mut acc = 0.0f32;
    for (i, val) in input.iter().enumerate() {
        acc += val;
        
        let val_abs = if acc < 0.0 { -acc } else { acc };
        let val_clamped = if val_abs > 1.0 { 1.0 } else { val_abs };
        let pixel = (val_clamped * 255.0) as u8;
        output[i] = if pixel >= 240 { 255 } else { pixel };
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn accumulate_and_map_sse2(input: &[f32], output: &mut [u8]) {
    let len = input.len();
    let mut i = 0;
    let mut running_acc = _mm_setzero_ps();
    let v_255 = _mm_set1_ps(255.0);
    let v_1_0 = _mm_set1_ps(1.0);
    let v_abs_mask = _mm_castsi128_ps(_mm_set1_epi32(0x7FFFFFFF));

    while i + 4 <= len {
        let delta = _mm_loadu_ps(input.as_ptr().add(i));

        // Prefix sum:
        // x = [d0, d1, d2, d3]
        // s1 = [0, d0, d1, d2] (shift right by 4 bytes = 1 float)
        let s1 = _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(delta), 4));
        let x = _mm_add_ps(delta, s1);
        
        // s2 = [0, 0, x0, x1] (shift right by 8 bytes = 2 floats)
        let s2 = _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 8));
        let prefix = _mm_add_ps(x, s2);

        let current_accs = _mm_add_ps(prefix, running_acc);

        // Update running_acc: broadcast last element (index 3)
        running_acc = _mm_shuffle_ps(current_accs, current_accs, 0xFF); 

        // Process
        let val_abs = _mm_and_ps(current_accs, v_abs_mask);
        let val_clamped = _mm_min_ps(val_abs, v_1_0);
        let val_scaled = _mm_mul_ps(val_clamped, v_255);
        let val_i32 = _mm_cvtps_epi32(val_scaled); // Round to nearest integer

        // Pack i32 -> i16 -> u8
        let val_i16 = _mm_packs_epi32(val_i32, _mm_setzero_si128());
        let val_u8 = _mm_packus_epi16(val_i16, _mm_setzero_si128());

        let pixel_val = _mm_cvtsi128_si32(val_u8);
        
        // Scalar fallback for the threshold logic (simpler than SIMD bitmasking for just 4 bytes)
        let mut bytes = pixel_val.to_le_bytes();
        // We only care about the first 4 bytes
        if bytes[0] >= 240 { bytes[0] = 255; }
        if bytes[1] >= 240 { bytes[1] = 255; }
        if bytes[2] >= 240 { bytes[2] = 255; }
        if bytes[3] >= 240 { bytes[3] = 255; }

        core::ptr::copy_nonoverlapping(bytes.as_ptr(), output.as_mut_ptr().add(i), 4);

        i += 4;
    }

    // Handle remainder
    let mut current_scalar_acc = _mm_cvtss_f32(running_acc);
    while i < len {
        current_scalar_acc += *input.get_unchecked(i);
        let val_abs = current_scalar_acc.abs();
        let val_clamped = if val_abs > 1.0 { 1.0 } else { val_abs };
        let pixel = (val_clamped * 255.0) as u8;
        *output.get_unchecked_mut(i) = if pixel >= 240 { 255 } else { pixel };
        i += 1;
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn accumulate_and_map_neon(input: &[f32], output: &mut [u8]) {
    let len = input.len();
    let mut i = 0;
    let mut running_acc = vdupq_n_f32(0.0);
    let v_255 = vdupq_n_f32(255.0);
    let v_1_0 = vdupq_n_f32(1.0);

    while i + 4 <= len {
        let delta = vld1q_f32(input.as_ptr().add(i));

        // Prefix sum
        // Shift logic using vextq (byte extraction)
        // Cast to u8 to use byte shift if needed, or use generic intrinsics if available.
        // Ideally: [0, d0, d1, d2]
        // In NEON, we can use vcombine + vget_low/high or vext.
        // vextq_f32(a, b, n) extracts from a|b.
        // To get [0, d0, d1, d2], we want last 1 of Zero and first 3 of Delta.
        // vextq_f32(zero, delta, 3).
        
        let zero = vdupq_n_f32(0.0);
        let s1 = vextq_f32(zero, delta, 3); 
        let x = vaddq_f32(delta, s1);

        let s2 = vextq_f32(zero, x, 2); // [0, 0, x0, x1]
        let prefix = vaddq_f32(x, s2);

        let current_accs = vaddq_f32(prefix, running_acc);

        // Broadcast last element for next iteration
        running_acc = vdupq_n_f32(vgetq_lane_f32(current_accs, 3));

        let val_abs = vabsq_f32(current_accs);
        let val_clamped = vminq_f32(val_abs, v_1_0);
        let val_scaled = vmulq_f32(val_clamped, v_255);
        
        // Convert to u32
        let val_u32 = vcvtq_u32_f32(val_scaled);

        // Pack u32 -> u16 -> u8
        let val_u16 = vqmovn_u32(val_u32); // 4x u16
        let val_u8_vec = vqmovn_u16(vcombine_u16(val_u16, val_u16)); // 8x u8 (bottom 4 are ours)
        
        let val_u32_lane = vget_lane_u32(vreinterpret_u32_u8(val_u8_vec), 0);
        let mut bytes = val_u32_lane.to_le_bytes();

        if bytes[0] >= 240 { bytes[0] = 255; }
        if bytes[1] >= 240 { bytes[1] = 255; }
        if bytes[2] >= 240 { bytes[2] = 255; }
        if bytes[3] >= 240 { bytes[3] = 255; }

        core::ptr::copy_nonoverlapping(bytes.as_ptr(), output.as_mut_ptr().add(i), 4);
        i += 4;
    }

    // Remainder
    let mut current_scalar_acc = vgetq_lane_f32(running_acc, 3);
    while i < len {
        current_scalar_acc += *input.get_unchecked(i);
        let val_abs = current_scalar_acc.abs();
        let val_clamped = if val_abs > 1.0 { 1.0 } else { val_abs };
        let pixel = (val_clamped * 255.0) as u8;
        *output.get_unchecked_mut(i) = if pixel >= 240 { 255 } else { pixel };
        i += 1;
    }
}
