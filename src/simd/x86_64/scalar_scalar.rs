use crate::ScalarF4E4;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Pack 15× i16 values into 17-bit slots (255 bits total)
///
/// Takes 15 i16 values and packs them into a 256-bit register where each value
/// occupies 17 bits (16 data bits + 1 bit for overflow/carry space).
///
/// Bit layout: [val0: 0-16][val1: 17-33][val2: 34-50]...[val14: 238-254][unused: 255]
#[allow(dead_code)]
#[inline]
unsafe fn pack_15_i16_to_17bit(values: &[i16; 15]) -> __m256i {
    // Convert i16 to i32 for easier bit manipulation
    let mut packed = [0u32; 8];

    // Pack values: each 17-bit value spans into u32 chunks
    // val0: bits 0-16
    packed[0] |= values[0] as u32 & 0x1FFFF;
    // val1: bits 17-33 (spans packed[0] and packed[1])
    packed[0] |= (values[1] as u32 & 0x7FFF) << 17;
    packed[1] |= (values[1] as u32 & 0x1FFFF) >> 15;
    // val2: bits 34-50
    packed[1] |= (values[2] as u32 & 0x3FFF) << 2;
    packed[1] |= ((values[2] as u32 & 0x1FFFF) >> 14) << 16;
    packed[2] |= (values[2] as u32 & 0x1FFFF) >> 30;

    // ... continue for all 15 values
    // TODO: Complete the bit packing for remaining values

    _mm256_loadu_si256(packed.as_ptr() as *const __m256i)
}

/// Unpack 15× 17-bit values from a 256-bit register back to i16
///
/// Extracts 15 values from 17-bit slots and converts back to i16.
/// Inverse of pack_15_i16_to_17bit().
#[allow(dead_code)]
#[inline]
unsafe fn unpack_17bit_to_15_i16(packed: __m256i) -> [i16; 15] {
    let mut unpacked = [0i16; 15];
    let bytes: [u32; 8] = core::mem::transmute(packed);

    // Extract val0: bits 0-16
    unpacked[0] = (bytes[0] & 0x1FFFF) as i16;
    // Extract val1: bits 17-33
    unpacked[1] = (((bytes[0] >> 17) | (bytes[1] << 15)) & 0x1FFFF) as i16;
    // Extract val2: bits 34-50
    unpacked[2] = (((bytes[1] >> 2) | (bytes[2] << 30)) & 0x1FFFF) as i16;

    // ... continue for all 15 values
    // TODO: Complete the bit unpacking for remaining values

    unpacked
}

/// Version 1: 8-op baseline using hardware i16→i32 unpack
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2_8op(
    a: &[ScalarF4E4; 8],
    b: &[ScalarF4E4; 8],
    result: &mut [ScalarF4E4; 8],
) {
    // Load 8× ScalarF4E4 as 256 bits (each ScalarF4E4 = 32 bits)
    // Memory layout: [frac0|exp0][frac1|exp1]...[frac7|exp7] (each pair is 32 bits)
    let a_vec = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
    let b_vec = _mm256_loadu_si256(b.as_ptr() as *const __m256i);

    // Extract fractions (lower 16 bits of each 32-bit lane, sign-extended to i32)
    // Shift left 16 to move fraction to upper position, then arithmetic shift right to sign-extend
    let a_frac = _mm256_srai_epi32(_mm256_slli_epi32(a_vec, 16), 16);
    let b_frac = _mm256_srai_epi32(_mm256_slli_epi32(b_vec, 16), 16);

    // Extract exponents (upper 16 bits of each 32-bit lane, sign-extended to i32)
    // Arithmetic shift right 16 extracts and sign-extends
    let a_exp = _mm256_srai_epi32(a_vec, 16);
    let b_exp = _mm256_srai_epi32(b_vec, 16);

    // Now we have:
    // - a_frac: 8× i32 sign-extended fractions from a
    // - a_exp: 8× i32 sign-extended exponents from a
    // Same for b

    // Step 1: Compare exponents to determine which fraction needs shifting
    let exp_diff = _mm256_sub_epi32(a_exp, b_exp);
    let b_gt_a = _mm256_cmpgt_epi32(b_exp, a_exp); // -1 where b > a

    // Step 2: Compute absolute shift amount
    // We need abs(exp_diff) to know how much to shift
    let shift_amount = _mm256_abs_epi32(exp_diff);

    // Step 3: Align fractions by shifting the one with smaller exponent
    // If a > b: shift b_frac right by (a_exp - b_exp)
    // If b > a: shift a_frac right by (b_exp - a_exp)
    // If equal: no shift needed

    // Arithmetic right shift (preserves sign bit for signed fractions)
    let a_frac_shifted = _mm256_srav_epi32(a_frac, shift_amount);
    let b_frac_shifted = _mm256_srav_epi32(b_frac, shift_amount);

    // Select which fraction to use based on exponent comparison
    // If a_exp > b_exp: use a_frac and b_frac_shifted
    // If b_exp > a_exp: use a_frac_shifted and b_frac
    // If equal: use a_frac and b_frac (no shift)
    let frac_a_final = _mm256_blendv_epi8(a_frac, a_frac_shifted, b_gt_a);
    let frac_b_final = _mm256_blendv_epi8(b_frac_shifted, b_frac, b_gt_a);

    // Step 4: Subtract aligned fractions
    let result_frac = _mm256_sub_epi32(frac_a_final, frac_b_final);

    // Step 5: Select result exponent (the larger one)
    let result_exp = _mm256_blendv_epi8(a_exp, b_exp, b_gt_a);

    // Step 6: Normalize (TODO: this needs lzcnt and more complex logic)
    // For now, pack without normalization as a starting point

    // Step 7: Pack back to i16 fractions and exponents
    // Truncate i32 back to i16 using packs (saturates)
    let result_frac_i16 = _mm256_packs_epi32(result_frac, result_frac);
    let result_exp_i16 = _mm256_packs_epi32(result_exp, result_exp);

    // Step 8: Interleave fractions and exponents back to [frac|exp] format
    // Unpack low 64 bits to interleave
    let result_lo = _mm256_unpacklo_epi16(result_frac_i16, result_exp_i16);
    let result_hi = _mm256_unpackhi_epi16(result_frac_i16, result_exp_i16);

    // Permute to correct lane order (packs puts lanes in wrong order)
    let result_vec =
        _mm256_permute4x64_epi64(_mm256_unpacklo_epi64(result_lo, result_hi), 0b11011000);

    _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, result_vec);

    // TODO: Handle edge cases (zero, normalization, ambiguous)
}

/// Version 2: 15-op using custom 17-bit packing with shifts/masks
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2_15op_shift(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // Process 15 ScalarF4E4 at a time
    let chunks = a.len() / 15;

    for i in 0..chunks {
        let idx = i * 15;

        // Extract 15 fractions (i16 for F4E4)
        let mut a_fracs = [0i16; 15];
        let mut b_fracs = [0i16; 15];
        for j in 0..15 {
            a_fracs[j] = a[idx + j].fraction;
            b_fracs[j] = b[idx + j].fraction;
        }

        // TODO: Pack into 17-bit format and subtract
        // let a_packed = pack_15_i16_to_17bit(&a_fracs);
        // let b_packed = pack_15_i16_to_17bit(&b_fracs);
        // let result_packed = subtract_packed_17bit(a_packed, b_packed);
        // let result_fracs = unpack_17bit_to_15_i16(result_packed);

        // Fallback to scalar for now
        for j in 0..15 {
            result[idx + j] = a[idx + j] - b[idx + j];
        }
    }

    // Handle remainder
    for i in (chunks * 15)..a.len() {
        result[i] = a[i] - b[i];
    }
}

/// Version 3: 15-op using BMI2 pdep/pext (requires modern CPU)
#[target_feature(enable = "avx2,bmi2")]
pub unsafe fn scalar_subtract_batch_avx2_15op_bmi2(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // Similar to 15-op shift version, but uses pdep/pext for packing
    // TODO: Implement with _pdep_u64() / _pext_u64()

    // Fallback to shift version for now
    scalar_subtract_batch_avx2_15op_shift(a, b, result);
}

/// Main AVX2 entry point - dispatches to best version
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // Process in chunks of 8
    let chunks = a.len() / 8;

    for i in 0..chunks {
        let idx = i * 8;

        // Convert slices to fixed arrays
        let a_chunk: &[ScalarF4E4; 8] = &a[idx..idx + 8].try_into().unwrap();
        let b_chunk: &[ScalarF4E4; 8] = &b[idx..idx + 8].try_into().unwrap();
        let result_chunk: &mut [ScalarF4E4; 8] = &mut result[idx..idx + 8].try_into().unwrap();

        scalar_subtract_batch_avx2_8op(a_chunk, b_chunk, result_chunk);
    }

    // Handle remainder with scalar fallback
    for i in (chunks * 8)..a.len() {
        result[i] = a[i] - b[i];
    }
}

/// Subtract 4x ScalarF4E4 using SSE4.2
///
/// Processes 4× i16 fractions and 4× i16 exponents per iteration.
#[target_feature(enable = "sse4.2")]
pub unsafe fn scalar_subtract_batch_sse42(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // TODO: Implement SSE4.2 version (4× ScalarF4E4)
    // Similar to AVX2 but with 128-bit registers
    scalar_subtract_fallback(a, b, result);
}

fn scalar_subtract_fallback(a: &[ScalarF4E4], b: &[ScalarF4E4], result: &mut [ScalarF4E4]) {
    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
