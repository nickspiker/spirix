// Spirix GPU Kernels - ScalarF4E4 Batch Subtraction
// Author: Nick Spiker fractaldecoder@proton.me
//
// Production GPU kernel for batch Spirix scalar subtraction using HIP/CUDA
// Based on assembly implementation from Spirix patent and Rust reference

#include <hip/hip_runtime.h>
#include <stdint.h>

// ScalarF4E4: F4 = 2^4 = 16-bit fraction, E4 = 2^4 = 16-bit exponent
// Total size: 32 bits (i16 + i16)
struct ScalarF4E4
{
    int16_t fraction; // Full 16-bit signed fraction (±32768)
    int16_t exponent; // Full 16-bit signed exponent (±32768)
};

// Special exponent value marking ambiguous/zero/vanished/exploded states
#define AMBIGUOUS_EXPONENT ((int16_t)0x8000) // -32768
#define FRACTION_BITS 16

// ============================================================================
// Reciprocal LUT - 256 entries in constant memory for fast division
// ============================================================================
// Lookup table for 1/x where x ∈ [1.0, 2.0) normalized range
// Indexed by bits [13:6] of normalized denominator (top 8 bits)
// Values scaled by 2^14 to preserve precision in fixed-point
__constant__ int16_t RECIPROCAL_LUT[256] = {
    0x4000, 0x3fc0, 0x3f80, 0x3f42, 0x3f03, 0x3ec6, 0x3e88, 0x3e4b,
    0x3e0f, 0x3dd3, 0x3d98, 0x3d5d, 0x3d22, 0x3ce8, 0x3cae, 0x3c75,
    0x3c3c, 0x3c03, 0x3bcb, 0x3b94, 0x3b5c, 0x3b25, 0x3aef, 0x3ab9,
    0x3a83, 0x3a4e, 0x3a19, 0x39e4, 0x39b0, 0x397c, 0x3949, 0x3916,
    0x38e3, 0x38b1, 0x387f, 0x384d, 0x381c, 0x37eb, 0x37ba, 0x3789,
    0x3759, 0x372a, 0x36fa, 0x36cb, 0x369d, 0x366e, 0x3640, 0x3612,
    0x35e5, 0x35b7, 0x358a, 0x355e, 0x3531, 0x3505, 0x34da, 0x34ae,
    0x3483, 0x3458, 0x342d, 0x3403, 0x33d9, 0x33af, 0x3385, 0x335c,
    0x3333, 0x330a, 0x32e1, 0x32b9, 0x3291, 0x3269, 0x3241, 0x321a,
    0x31f3, 0x31cc, 0x31a6, 0x317f, 0x3159, 0x3133, 0x310d, 0x30e8,
    0x30c3, 0x309e, 0x3079, 0x3054, 0x3030, 0x300c, 0x2fe8, 0x2fc4,
    0x2fa0, 0x2f7d, 0x2f5a, 0x2f37, 0x2f14, 0x2ef2, 0x2ecf, 0x2ead,
    0x2e8b, 0x2e69, 0x2e48, 0x2e26, 0x2e05, 0x2de4, 0x2dc3, 0x2da3,
    0x2d82, 0x2d62, 0x2d42, 0x2d22, 0x2d02, 0x2ce3, 0x2cc3, 0x2ca4,
    0x2c85, 0x2c66, 0x2c47, 0x2c29, 0x2c0b, 0x2bec, 0x2bce, 0x2bb0,
    0x2b93, 0x2b75, 0x2b58, 0x2b3a, 0x2b1d, 0x2b00, 0x2ae3, 0x2ac7,
    0x2aaa, 0x2a8e, 0x2a72, 0x2a55, 0x2a3a, 0x2a1e, 0x2a02, 0x29e7,
    0x29cb, 0x29b0, 0x2995, 0x297a, 0x295f, 0x2944, 0x292a, 0x2910,
    0x28f5, 0x28db, 0x28c1, 0x28a7, 0x288d, 0x2874, 0x285a, 0x2841,
    0x2828, 0x280f, 0x27f6, 0x27dd, 0x27c4, 0x27ab, 0x2793, 0x277a,
    0x2762, 0x274a, 0x2732, 0x271a, 0x2702, 0x26ea, 0x26d3, 0x26bb,
    0x26a4, 0x268c, 0x2675, 0x265e, 0x2647, 0x2630, 0x261a, 0x2603,
    0x25ed, 0x25d6, 0x25c0, 0x25aa, 0x2593, 0x257d, 0x2568, 0x2552,
    0x253c, 0x2526, 0x2511, 0x24fb, 0x24e6, 0x24d1, 0x24bc, 0x24a7,
    0x2492, 0x247d, 0x2468, 0x2454, 0x243f, 0x242a, 0x2416, 0x2402,
    0x23ee, 0x23d9, 0x23c5, 0x23b1, 0x239e, 0x238a, 0x2376, 0x2362,
    0x234f, 0x233c, 0x2328, 0x2315, 0x2302, 0x22ef, 0x22dc, 0x22c9,
    0x22b6, 0x22a3, 0x2290, 0x227e, 0x226b, 0x2259, 0x2246, 0x2234,
    0x2222, 0x220f, 0x21fd, 0x21eb, 0x21d9, 0x21c8, 0x21b6, 0x21a4,
    0x2192, 0x2181, 0x216f, 0x215e, 0x214d, 0x213b, 0x212a, 0x2119,
    0x2108, 0x20f7, 0x20e6, 0x20d5, 0x20c4, 0x20b3, 0x20a3, 0x2092,
    0x2082, 0x2071, 0x2061, 0x2050, 0x2040, 0x2030, 0x2020, 0x2010,
};

// ============================================================================
// Scalar Subtraction Kernel: a - b
// ============================================================================
// Processes batch of ScalarF4E4 subtraction with full normalization
// Uses native GPU __clz() for leading zero count
// All threads execute same instruction sequence (SIMT-friendly)
__global__ void scalar_subtract_kernel(
    const ScalarF4E4 *__restrict__ a,
    const ScalarF4E4 *__restrict__ b,
    ScalarF4E4 *__restrict__ result,
    int n)
{
    int idx, lz, lo, leading;
    int32_t a_frac, b_frac, a_exp, b_exp;
    int32_t exp_diff, abs_diff, big_a, big_b;
    int32_t result_frac, result_exp, temp_exp;
    int32_t mask_a_larger, mask_ambiguous, mask_negligible;
    int32_t mask_zero, mask_underflow, mask_valid;
    int32_t sub_result, shift_n1, shift_n2, norm_n1, norm_n2;

    // Thread index
    idx = blockIdx.x * blockDim.x + threadIdx.x;
    mask_valid = -(idx < n);

    // Load inputs (i16 → i32 with sign extension)
    a_frac = (int32_t)a[idx].fraction;
    b_frac = (int32_t)b[idx].fraction;
    a_exp = (int32_t)a[idx].exponent;
    b_exp = (int32_t)b[idx].exponent;

    // Check for ambiguous inputs
    mask_ambiguous = -((a_exp == AMBIGUOUS_EXPONENT) | (b_exp == AMBIGUOUS_EXPONENT));

    // Exponent alignment
    exp_diff = a_exp - b_exp;
    abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;
    mask_a_larger = -(exp_diff >= 0);
    mask_negligible = -(abs_diff >= FRACTION_BITS);

    // Fraction alignment and subtraction
    big_a = a_frac << abs_diff;
    big_b = b_frac << abs_diff;

    sub_result = (mask_a_larger & (big_a - b_frac)) | (~mask_a_larger & (a_frac - big_b));
    result_exp = (mask_a_larger & b_exp) | (~mask_a_larger & a_exp);

    // Sign-extend subtraction result from 16-bit to 32-bit
    sub_result = (sub_result << 16) >> 16;

    // Check for zero
    mask_zero = -(sub_result == 0);

    // Count leading same bits
    lz = __clz(sub_result);
    lo = __clz(~sub_result);
    leading = (lz > lo) ? lz : lo;

    // Calculate adjusted exponent: result_exp + (FRACTION_BITS - leading) + 1
    temp_exp = result_exp + 17 - leading;

    // Check for underflow
    mask_underflow = -(temp_exp <= AMBIGUOUS_EXPONENT);

    // Normalize fraction (N-1 and N-2)
    shift_n1 = leading - 1;
    shift_n2 = leading - 2;
    norm_n1 = (sub_result << shift_n1) >> 16;
    norm_n2 = (sub_result << shift_n2) >> 16;

    result_frac = (mask_underflow & norm_n2) | (~mask_underflow & norm_n1);
    result_exp = (mask_underflow & AMBIGUOUS_EXPONENT) | (~mask_underflow & temp_exp);

    // Final result selection (cascading priority)
    result_frac = (mask_ambiguous & 0) |
                  (~mask_ambiguous & mask_negligible & mask_a_larger & a_frac) |
                  (~mask_ambiguous & mask_negligible & ~mask_a_larger & (-b_frac)) |
                  (~mask_ambiguous & ~mask_negligible & mask_zero & 0) |
                  (~mask_ambiguous & ~mask_negligible & ~mask_zero & result_frac);

    result_exp = (mask_ambiguous & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & mask_negligible & mask_a_larger & a_exp) |
                 (~mask_ambiguous & mask_negligible & ~mask_a_larger & b_exp) |
                 (~mask_ambiguous & ~mask_negligible & mask_zero & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & ~mask_negligible & ~mask_zero & result_exp);

    // Write result (only valid threads)
    result[idx].fraction = (int16_t)(result_frac & mask_valid);
    result[idx].exponent = (int16_t)(result_exp & mask_valid);
}

// ============================================================================
// Scalar Addition Kernel: a + b
// ============================================================================
// Identical to subtraction except: addition instead of subtraction (line 65)
// Overflow from addition is naturally handled by normalization logic
__global__ void scalar_add_kernel(
    const ScalarF4E4 *__restrict__ a,
    const ScalarF4E4 *__restrict__ b,
    ScalarF4E4 *__restrict__ result,
    int n)
{
    int idx, lz, lo, leading;
    int32_t a_frac, b_frac, a_exp, b_exp;
    int32_t exp_diff, abs_diff, big_a, big_b;
    int32_t result_frac, result_exp, temp_exp;
    int32_t mask_a_larger, mask_ambiguous, mask_negligible;
    int32_t mask_zero, mask_underflow, mask_valid;
    int32_t add_result, shift_n1, shift_n2, norm_n1, norm_n2;

    // Thread index
    idx = blockIdx.x * blockDim.x + threadIdx.x;
    mask_valid = -(idx < n);

    // Load inputs (i16 → i32 with sign extension)
    a_frac = (int32_t)a[idx].fraction;
    b_frac = (int32_t)b[idx].fraction;
    a_exp = (int32_t)a[idx].exponent;
    b_exp = (int32_t)b[idx].exponent;

    // Check for ambiguous inputs
    mask_ambiguous = -((a_exp == AMBIGUOUS_EXPONENT) | (b_exp == AMBIGUOUS_EXPONENT));

    // Exponent alignment
    exp_diff = a_exp - b_exp;
    abs_diff = (exp_diff < 0) ? -exp_diff : exp_diff;
    mask_a_larger = -(exp_diff >= 0);
    mask_negligible = -(abs_diff >= FRACTION_BITS);

    // Fraction alignment and ADDITION (only difference from subtraction!)
    big_a = a_frac << abs_diff;
    big_b = b_frac << abs_diff;

    add_result = (mask_a_larger & (big_a + b_frac)) | (~mask_a_larger & (a_frac + big_b));
    result_exp = (mask_a_larger & b_exp) | (~mask_a_larger & a_exp);

    // Sign-extend result from 16-bit to 32-bit
    add_result = (add_result << 16) >> 16;

    // Check for zero
    mask_zero = -(add_result == 0);

    // Count leading same bits (handles overflow naturally)
    lz = __clz(add_result);
    lo = __clz(~add_result);
    leading = (lz > lo) ? lz : lo;

    // Calculate adjusted exponent
    temp_exp = result_exp + 17 - leading;

    // Check for underflow
    mask_underflow = -(temp_exp <= AMBIGUOUS_EXPONENT);

    // Normalize fraction (N-1 and N-2)
    shift_n1 = leading - 1;
    shift_n2 = leading - 2;
    norm_n1 = (add_result << shift_n1) >> 16;
    norm_n2 = (add_result << shift_n2) >> 16;

    result_frac = (mask_underflow & norm_n2) | (~mask_underflow & norm_n1);
    result_exp = (mask_underflow & AMBIGUOUS_EXPONENT) | (~mask_underflow & temp_exp);

    // Final result selection (cascading priority)
    // For addition: if negligible, return the larger value unchanged
    result_frac = (mask_ambiguous & 0) |
                  (~mask_ambiguous & mask_negligible & mask_a_larger & a_frac) |
                  (~mask_ambiguous & mask_negligible & ~mask_a_larger & b_frac) |
                  (~mask_ambiguous & ~mask_negligible & mask_zero & 0) |
                  (~mask_ambiguous & ~mask_negligible & ~mask_zero & result_frac);

    result_exp = (mask_ambiguous & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & mask_negligible & mask_a_larger & a_exp) |
                 (~mask_ambiguous & mask_negligible & ~mask_a_larger & b_exp) |
                 (~mask_ambiguous & ~mask_negligible & mask_zero & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & ~mask_negligible & ~mask_zero & result_exp);

    // Write result (only valid threads)
    result[idx].fraction = (int16_t)(result_frac & mask_valid);
    result[idx].exponent = (int16_t)(result_exp & mask_valid);
}

// ============================================================================
// Newton-Raphson Division Helper: a / b using LUT + 2 iterations
// ============================================================================
__device__ int32_t newton_raphson_divide(int32_t numerator, int32_t denominator)
{
    // Get absolute values for computation
    int32_t abs_numer = (numerator < 0) ? -numerator : numerator;
    int32_t abs_denom = (denominator < 0) ? -denominator : denominator;
    bool result_negative = (numerator < 0) != (denominator < 0);

    // LUT lookup: index by bits [13:6] of normalized denominator
    int index = (abs_denom >> 6) & 0xFF;
    int32_t recip = (int32_t)RECIPROCAL_LUT[index];  // 2^14 scale

    // Newton-Raphson refinement: 2 iterations
    // Formula: y_new = y * (2 - b*y)
    // All values in 2^14 fixed-point scale
    for (int iter = 0; iter < 2; iter++)
    {
        int32_t prod = (abs_denom * recip) >> 14;  // b*y in 2^14 scale
        int32_t error = 0x8000 - prod;              // (2.0 - b*y) in 2^14 scale
        recip = ((int64_t)recip * (int64_t)error) >> 14;
    }

    // Multiply numerator by reciprocal: a * (1/b)
    // Both in 2^14 scale, shift by 12 for final result
    int64_t product = (int64_t)abs_numer * (int64_t)recip;
    int32_t quotient = (int32_t)(product >> 12);

    return result_negative ? -quotient : quotient;
}

// ============================================================================
// Scalar Division Kernel: a / b
// ============================================================================
// Uses Newton-Raphson with 256-entry LUT for fast reciprocal approximation
__global__ void scalar_divide_kernel(
    const ScalarF4E4 *__restrict__ a,
    const ScalarF4E4 *__restrict__ b,
    ScalarF4E4 *__restrict__ result,
    int n)
{
    int idx, lz, lo, leading;
    int32_t a_frac, b_frac, a_exp, b_exp;
    int32_t quotient, result_frac, result_exp;
    int32_t mask_div_by_zero, mask_zero_dividend, mask_valid;
    int32_t shift, expo_adjust, temp_exp;

    // Thread index
    idx = blockIdx.x * blockDim.x + threadIdx.x;
    mask_valid = -(idx < n);

    // Load inputs
    a_frac = (int32_t)a[idx].fraction;
    b_frac = (int32_t)b[idx].fraction;
    a_exp = (int32_t)a[idx].exponent;
    b_exp = (int32_t)b[idx].exponent;

    // Check edge cases
    mask_div_by_zero = -(b_frac == 0);
    mask_zero_dividend = -(a_frac == 0);

    // Newton-Raphson division (only if both non-zero)
    quotient = newton_raphson_divide(a_frac, b_frac);

    // Count leading same bits for normalization
    lz = __clz(quotient);
    lo = __clz(~quotient);
    leading = (lz > lo) ? lz : lo;

    // Normalization shift (leading - 1 for N-1 position)
    shift = leading - 1;

    // Normalize quotient and extract fraction
    int32_t normalized = quotient << shift;
    result_frac = normalized >> 16;

    // Exponent calculation: a_exp - b_exp - expo_adjust
    // expo_adjust = shift - 15 (accounts for normalization)
    expo_adjust = shift - 15;
    temp_exp = a_exp - b_exp - expo_adjust;

    // Final result selection
    result_frac = (mask_div_by_zero & 0x7FFF) |         // Infinity fraction
                  (mask_zero_dividend & 0) |             // Zero fraction
                  (~(mask_div_by_zero | mask_zero_dividend) & result_frac);

    result_exp = (mask_div_by_zero & 127) |             // Infinity exponent (large)
                 (mask_zero_dividend & AMBIGUOUS_EXPONENT) |  // Zero exponent
                 (~(mask_div_by_zero | mask_zero_dividend) & temp_exp);

    // Write result (only valid threads)
    result[idx].fraction = (int16_t)(result_frac & mask_valid);
    result[idx].exponent = (int16_t)(result_exp & mask_valid);
}

// ============================================================================
// Bitwise Integer Square Root Helper (IEEE-754 free)
// ============================================================================
// Computes integer square root using binary search with fully unrolled loop
// Returns 16-bit result from 32-bit input (sufficient for ScalarF4E4 fractions)
__device__ uint32_t bitwise_sqrt_u32(uint32_t radicand)
{
    uint32_t bit = 1u << 15;  // Start from middle bit for 16-bit result
    uint32_t result = 0;

    // Fully unrolled 16-iteration binary search
    #pragma unroll
    for (int i = 0; i < 16; i++)
    {
        uint32_t guess = result | bit;
        // Use branchless: result = (guess² <= radicand) ? guess : result
        // Compiler will optimize this to conditional move
        if ((uint64_t)guess * (uint64_t)guess <= (uint64_t)radicand)
        {
            result = guess;
        }
        bit >>= 1;
    }

    return result;
}

// ============================================================================
// Scalar Square Root Kernel: sqrt(a)
// ============================================================================
// Computes square root using bitwise integer sqrt + exponent halving
// Handles negative inputs (returns ambiguous) and zero correctly
__global__ void scalar_sqrt_kernel(
    const ScalarF4E4 *__restrict__ a,
    ScalarF4E4 *__restrict__ result,
    int n)
{
    int idx, lz;
    int32_t a_frac, a_exp;
    int32_t result_frac, result_exp;
    int32_t mask_zero, mask_negative, mask_valid;
    int32_t exponent, even, shift_adjust;
    uint32_t value, shifted_value, sqrt_result, leading_zeros;

    // Thread index
    idx = blockIdx.x * blockDim.x + threadIdx.x;
    mask_valid = -(idx < n);

    // Load input
    a_frac = (int32_t)a[idx].fraction;
    a_exp = (int32_t)a[idx].exponent;

    // Check edge cases
    mask_zero = -(a_frac == 0);
    mask_negative = -(a_frac < 0);

    // Exponent calculation: divide by 2, handle odd exponents
    // For odd negative exponents, we need floor division
    exponent = a_exp / 2;
    even = a_exp & 1;  // 1 if odd, 0 if even
    exponent = exponent + even;  // Adjust for odd exponents

    // Correct for negative odd exponents: floor(neg/2) needs -1
    int32_t neg_odd_adjust = -((a_exp < 0) & (even != 0));
    exponent = exponent + neg_odd_adjust;

    // Prepare fraction for sqrt: shift to position
    // For even exponents: shift by 17 bits
    // For odd exponents: shift by 16 bits (17 - 1)
    value = (uint32_t)(uint16_t)a_frac;  // Convert to unsigned
    shifted_value = value << (17 - even);

    // Compute sqrt using bitwise method
    sqrt_result = bitwise_sqrt_u32(shifted_value);

    // Normalize the result
    leading_zeros = __clz(sqrt_result) - 1;
    sqrt_result = sqrt_result << leading_zeros;
    shift_adjust = (int32_t)leading_zeros - 15;
    exponent = exponent + shift_adjust;

    // Extract normalized fraction (top 16 bits)
    result_frac = (int32_t)(sqrt_result >> 16);
    result_exp = exponent;

    // Final result selection
    result_frac = (mask_zero & 0) |            // Zero input → zero fraction
                  (mask_negative & 0) |         // Negative input → zero fraction (undefined)
                  (~(mask_zero | mask_negative) & result_frac);

    result_exp = (mask_zero & AMBIGUOUS_EXPONENT) |        // Zero input → ambiguous
                 (mask_negative & AMBIGUOUS_EXPONENT) |    // Negative input → ambiguous (undefined)
                 (~(mask_zero | mask_negative) & result_exp);

    // Write result (only valid threads)
    result[idx].fraction = (int16_t)(result_frac & mask_valid);
    result[idx].exponent = (int16_t)(result_exp & mask_valid);
}

// ============================================================================
// Scalar Multiplication Kernel: a * b
// ============================================================================
// Multiplies fractions using wider integers, normalizes, adds exponents
__global__ void scalar_multiply_kernel(
    const ScalarF4E4 *__restrict__ a,
    const ScalarF4E4 *__restrict__ b,
    ScalarF4E4 *__restrict__ result,
    int n)
{
    int idx, lz, lo, leading;
    int32_t a_frac, b_frac, a_exp, b_exp;
    int32_t product, result_frac, result_exp;
    int32_t mask_ambiguous, mask_zero, mask_overflow, mask_underflow, mask_valid;
    int32_t expo_adjust, shift, temp_exp;

    // Thread index
    idx = blockIdx.x * blockDim.x + threadIdx.x;
    mask_valid = -(idx < n);

    // Load inputs (i16 → i32 with sign extension)
    a_frac = (int32_t)a[idx].fraction;
    b_frac = (int32_t)b[idx].fraction;
    a_exp = (int32_t)a[idx].exponent;
    b_exp = (int32_t)b[idx].exponent;

    // Check for ambiguous inputs
    mask_ambiguous = -((a_exp == AMBIGUOUS_EXPONENT) | (b_exp == AMBIGUOUS_EXPONENT));

    // Multiply fractions (i16 × i16 = i32 product)
    product = a_frac * b_frac;

    // Check for zero
    mask_zero = -(product == 0);

    // Count leading same bits
    lz = __clz(product);
    lo = __clz(~product);
    leading = (lz > lo) ? lz : lo;

    // Exponent adjustment: leading - 2 (accounts for double-width product)
    expo_adjust = leading - 2;
    shift = expo_adjust + 1;

    // Normalize: shift product and extract high half
    int32_t normalized_product = product << shift;
    result_frac = normalized_product >> FRACTION_BITS;

    // Add exponents and subtract adjustment
    temp_exp = a_exp + b_exp - expo_adjust;

    // Check for overflow/underflow
    mask_overflow = -(temp_exp > (int32_t)32767);   // i16 max
    mask_underflow = -(temp_exp <= AMBIGUOUS_EXPONENT);

    // Set exponent (overflow and underflow both → AMBIGUOUS)
    result_exp = ((mask_overflow | mask_underflow) & AMBIGUOUS_EXPONENT) |
                 (~(mask_overflow | mask_underflow) & temp_exp);

    // For underflow, shift fraction right by 1
    result_frac = (mask_underflow & (result_frac >> 1)) |
                  (~mask_underflow & result_frac);

    // Final result selection
    result_frac = (mask_ambiguous & 0) |
                  (~mask_ambiguous & mask_zero & 0) |
                  (~mask_ambiguous & ~mask_zero & result_frac);

    result_exp = (mask_ambiguous & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & mask_zero & AMBIGUOUS_EXPONENT) |
                 (~mask_ambiguous & ~mask_zero & result_exp);

    // Write result (only valid threads)
    result[idx].fraction = (int16_t)(result_frac & mask_valid);
    result[idx].exponent = (int16_t)(result_exp & mask_valid);
}

// ============================================================================
// Host API Wrappers
// ============================================================================

extern "C" void spirix_scalar_subtract_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(scalar_subtract_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_result, n);
}

extern "C" void spirix_scalar_add_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(scalar_add_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_result, n);
}

extern "C" void spirix_scalar_multiply_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(scalar_multiply_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_result, n);
}

extern "C" void spirix_scalar_divide_gpu(
    const ScalarF4E4 *d_a,
    const ScalarF4E4 *d_b,
    ScalarF4E4 *d_result,
    int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(scalar_divide_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_b, d_result, n);
}

extern "C" void spirix_scalar_sqrt_gpu(
    const ScalarF4E4 *d_a,
    ScalarF4E4 *d_result,
    int n)
{
    int threads = 256;
    int blocks = (n + threads - 1) / threads;
    hipLaunchKernelGGL(scalar_sqrt_kernel, dim3(blocks), dim3(threads), 0, 0,
                       d_a, d_result, n);
}
