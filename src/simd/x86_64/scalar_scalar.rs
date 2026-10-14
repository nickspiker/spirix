use crate::{Scalar, ScalarF4E4};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// The vector kernel reads each ScalarF4E4 as one little-endian 32-bit word with the fraction in the low half and the exponent in the high half.
///
/// `Scalar` is not `repr(C)`, so this is checked at compile time and the batch functions fall back to element-wise scalar subtract if the compiler ever lays it out differently.
const PACKED_LAYOUT: bool = core::mem::size_of::<ScalarF4E4>() == 4
    && core::mem::offset_of!(Scalar<i16, i16>, fraction) == 0
    && core::mem::offset_of!(Scalar<i16, i16>, exponent) == 2;

/// Subtracts 8x ScalarF4E4 with AVX2, bit-identical to `a[i] - b[i]`.
///
/// The vector path mirrors `scalar_subtract_scalar` for the common case: both operands normal (stored exponent nonzero), the smaller operand either aligned by a shift of at most FRAC-2 or dropped entirely, and the normalized result landing inside the exponent cycle.
///
/// Lanes outside that case (any Zero, Infinity, exploded, vanished or undefined operand, a result that explodes or vanishes, or negating a dropped subtrahend sitting on a ±1.0 fraction boundary) are recomputed with the scalar operator after the vector store.
///
/// # Safety
///
/// Requires AVX2.
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2_8op(
    a: &[ScalarF4E4; 8],
    b: &[ScalarF4E4; 8],
    result: &mut [ScalarF4E4; 8],
) {
    if !PACKED_LAYOUT {
        for i in 0..8 {
            result[i] = a[i] - b[i];
        }
        return;
    }

    // Each 32-bit lane holds [exponent:16 | fraction:16].
    let va = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
    let vb = _mm256_loadu_si256(b.as_ptr() as *const __m256i);
    let zero = _mm256_setzero_si256();
    let low16 = _mm256_set1_epi32(0xFFFF);

    // Stored exponents zero-extended, so a signed 32-bit compare is the unsigned cycle-position compare the scalar code uses.
    let ea = _mm256_srli_epi32(va, 16);
    let eb = _mm256_srli_epi32(vb, 16);
    // Fractions sign-extended, then inflated for a normal operand by flipping the upper half (`inflate(true)`).
    let fa = _mm256_srai_epi32(_mm256_slli_epi32(va, 16), 16);
    let fb = _mm256_srai_epi32(_mm256_slli_epi32(vb, 16), 16);
    let flip = _mm256_set1_epi32(0xFFFF_0000u32 as i32);
    let ia = _mm256_xor_si256(fa, flip);
    let ib = _mm256_xor_si256(fb, flip);

    // AMBIGUOUS exponent (0) marks every non-normal class; those lanes go to the scalar operator.
    let not_normal = _mm256_or_si256(_mm256_cmpeq_epi32(ea, zero), _mm256_cmpeq_epi32(eb, zero));

    // big_is_self is a strict unsigned greater-than, so equal exponents treat b as big, matching the scalar code.
    let a_big = _mm256_cmpgt_epi32(ea, eb);
    let diff = _mm256_abs_epi32(_mm256_sub_epi32(ea, eb));
    // The scalar code drops the smaller operand when the i16 wrapped difference is negative or the shift is at least FRAC-1 = 15, which together is diff > 14.
    let lost = _mm256_cmpgt_epi32(diff, _mm256_set1_epi32(14));

    // Shift whichever operand is big left by diff, then subtract in operand order: big_f - small_f when a is big, small_f - big_f otherwise, which is always a' - b'.
    let shift_a = _mm256_and_si256(diff, a_big);
    let shift_b = _mm256_andnot_si256(a_big, diff);
    let r = _mm256_sub_epi32(
        _mm256_sllv_epi32(ia, shift_a),
        _mm256_sllv_epi32(ib, shift_b),
    );
    let r_zero = _mm256_cmpeq_epi32(r, zero);

    // leading_same(r): fold the sign away, smear the top set bit down, isolate it, and read its position from the exact float conversion of that power of two.
    let mut x = _mm256_xor_si256(r, _mm256_srai_epi32(r, 31));
    x = _mm256_or_si256(x, _mm256_srli_epi32(x, 1));
    x = _mm256_or_si256(x, _mm256_srli_epi32(x, 2));
    x = _mm256_or_si256(x, _mm256_srli_epi32(x, 4));
    x = _mm256_or_si256(x, _mm256_srli_epi32(x, 8));
    x = _mm256_or_si256(x, _mm256_srli_epi32(x, 16));
    let top = _mm256_andnot_si256(_mm256_srli_epi32(x, 1), x);
    let top_exp = _mm256_srli_epi32(_mm256_castps_si256(_mm256_cvtepi32_ps(top)), 23);
    // top_exp = 127 + bit position, or 0 when r is all sign bits; clamping at 32 covers the latter.
    let leading = _mm256_min_epi32(
        _mm256_sub_epi32(_mm256_set1_epi32(31 + 127), top_exp),
        _mm256_set1_epi32(32),
    );

    // delta = FRAC - leading, new exponent = small exponent + delta, which must stay inside the normal range 1..=0xFFFF.
    let delta = _mm256_sub_epi32(_mm256_set1_epi32(16), leading);
    let small_e = _mm256_blendv_epi8(ea, eb, a_big);
    let offset = _mm256_add_epi32(small_e, delta);
    let escapes = _mm256_or_si256(
        _mm256_cmpgt_epi32(_mm256_set1_epi32(1), offset),
        _mm256_cmpgt_epi32(offset, low16),
    );

    // Canonical fraction is r shifted by leading - FRAC: left by -delta when positive, arithmetic right by delta otherwise.
    let shl = _mm256_max_epi32(_mm256_sub_epi32(zero, delta), zero);
    let shr = _mm256_max_epi32(delta, zero);
    let canonical = _mm256_srav_epi32(_mm256_sllv_epi32(r, shl), shr);
    let main_out = _mm256_or_si256(
        _mm256_slli_epi32(offset, 16),
        _mm256_and_si256(canonical, low16),
    );
    // An exact zero difference is Spirix Zero, which is all-zero bits.
    let main_out = _mm256_andnot_si256(r_zero, main_out);

    // Dropped smaller operand: a when a is big, otherwise -b, which for a normal is wrapping_neg of the fraction except at the ±1.0 boundary fractions 0x8000 and 0x0000.
    let neg_b = _mm256_or_si256(
        _mm256_slli_epi32(eb, 16),
        _mm256_and_si256(_mm256_sub_epi32(zero, fb), low16),
    );
    let lost_out = _mm256_blendv_epi8(neg_b, va, a_big);
    let out = _mm256_blendv_epi8(main_out, lost_out, lost);

    let b_boundary = _mm256_or_si256(
        _mm256_cmpeq_epi32(fb, zero),
        _mm256_cmpeq_epi32(fb, _mm256_set1_epi32(i16::MIN as i32)),
    );
    let lost_slow = _mm256_andnot_si256(a_big, _mm256_and_si256(lost, b_boundary));
    let main_slow = _mm256_andnot_si256(lost, _mm256_andnot_si256(r_zero, escapes));
    let slow = _mm256_or_si256(not_normal, _mm256_or_si256(lost_slow, main_slow));

    _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, out);

    let mut slow_bits = _mm256_movemask_ps(_mm256_castsi256_ps(slow)) as u32;
    while slow_bits != 0 {
        let i = slow_bits.trailing_zeros() as usize;
        result[i] = a[i] - b[i];
        slow_bits &= slow_bits - 1;
    }
}

/// Subtracts arrays of ScalarF4E4 with AVX2, 8 lanes at a time, bit-identical to `a[i] - b[i]`.
///
/// # Safety
///
/// Requires AVX2.
///
/// # Panics
///
/// Panics if the three slices differ in length.
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    let a_chunks = a.chunks_exact(8);
    let b_chunks = b.chunks_exact(8);
    let a_tail = a_chunks.remainder();
    let b_tail = b_chunks.remainder();
    let mut r_chunks = result.chunks_exact_mut(8);
    for ((ac, bc), rc) in a_chunks.zip(b_chunks).zip(&mut r_chunks) {
        // Borrow each chunk as an array reference; converting the slice by value would copy it and discard the results.
        let ac: &[ScalarF4E4; 8] = ac.try_into().unwrap();
        let bc: &[ScalarF4E4; 8] = bc.try_into().unwrap();
        let rc: &mut [ScalarF4E4; 8] = rc.try_into().unwrap();
        scalar_subtract_batch_avx2_8op(ac, bc, rc);
    }
    scalar_subtract_fallback(a_tail, b_tail, r_chunks.into_remainder());
}

/// Subtracts arrays of ScalarF4E4, intended as a 15-lane packed AVX2 variant.
///
/// # Safety
///
/// Requires AVX2.
#[target_feature(enable = "avx2")]
pub unsafe fn scalar_subtract_batch_avx2_15op_shift(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // The 17-bit packed variant was never finished, so this is element-wise scalar subtract to stay bit-identical.
    scalar_subtract_fallback(a, b, result);
}

/// Subtracts arrays of ScalarF4E4, intended as a 15-lane packed AVX2 variant using BMI2 pdep/pext.
///
/// # Safety
///
/// Requires AVX2 and BMI2.
#[target_feature(enable = "avx2,bmi2")]
pub unsafe fn scalar_subtract_batch_avx2_15op_bmi2(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // The pdep/pext packed variant was never finished, so this is element-wise scalar subtract to stay bit-identical.
    scalar_subtract_fallback(a, b, result);
}

/// Subtracts arrays of ScalarF4E4, bit-identical to `a[i] - b[i]`.
///
/// # Safety
///
/// Requires SSE4.2.
#[target_feature(enable = "sse4.2")]
pub unsafe fn scalar_subtract_batch_sse42(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // SSE4.2 has no per-lane variable shifts for the alignment and normalize steps, so this is element-wise scalar subtract.
    scalar_subtract_fallback(a, b, result);
}

fn scalar_subtract_fallback(a: &[ScalarF4E4], b: &[ScalarF4E4], result: &mut [ScalarF4E4]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());
    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
