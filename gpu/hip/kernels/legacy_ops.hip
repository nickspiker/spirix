// Spirix GPU Operations - IEEE-754 Free Implementation
// Core GPU kernels for ScalarF4E4 arithmetic
#ifndef SPIRIX_OPS_HIP
#define SPIRIX_OPS_HIP

#include <hip/hip_runtime.h>
#include <stdint.h>

// ScalarF4E4: 16-bit fraction + 16-bit exponent
struct ScalarF4E4 {
    int16_t fraction;
    int16_t exponent;
};

// Constants for ScalarF4E4
#define AMBIGUOUS_EXPONENT -128
#define UNDEFINED_PREFIX -32768

// ============================================================================
// RECIPROCAL LUT - Constant Memory for Fast Access
// ============================================================================
// 256-entry reciprocal table for normalized fractions [1.0, 2.0)
// Indexed by bits[13:6] of denominator
// Values scaled by 2^14 to fit in int16_t
__constant__ int16_t RECIPROCAL_LUT_16[256] = {
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
// BITWISE INTEGER DIVISION - IEEE-754 FREE
// ============================================================================

__device__ int32_t bitwise_div_i32(int32_t dividend, int32_t divisor) {
    if (divisor == 0) {
        return (dividend >= 0) ? 0x7FFFFFFF : (int32_t)0x80000000;
    }

    bool negative = (dividend < 0) != (divisor < 0);

    uint32_t abs_dividend = (dividend < 0) ? -dividend : dividend;
    uint32_t abs_divisor = (divisor < 0) ? -divisor : divisor;

    uint32_t quotient = 0;
    uint32_t remainder = 0;

    // Fully unrolled - 32 iterations, no early exit for pipeline efficiency
    #pragma unroll
    for (int i = 31; i >= 0; i--) {
        remainder = (remainder << 1) | ((abs_dividend >> i) & 1);
        uint32_t sub = remainder - abs_divisor;
        bool fits = (sub <= remainder);
        quotient = (quotient << 1) | (fits ? 1 : 0);
        remainder = fits ? sub : remainder;
    }

    return negative ? -(int32_t)quotient : (int32_t)quotient;
}

// 16-bit integer division
__device__ int16_t bitwise_div_i16(int16_t dividend, int16_t divisor) {
    if (divisor == 0) {
        return (dividend >= 0) ? 0x7FFF : (int16_t)0x8000;
    }

    bool negative = (dividend < 0) != (divisor < 0);

    uint16_t abs_dividend = (dividend < 0) ? -dividend : dividend;
    uint16_t abs_divisor = (divisor < 0) ? -divisor : divisor;

    uint16_t quotient = 0;
    uint16_t remainder = 0;

    #pragma unroll
    for (int i = 15; i >= 0; i--) {
        remainder = (remainder << 1) | ((abs_dividend >> i) & 1);
        uint16_t sub = remainder - abs_divisor;
        bool fits = (sub <= remainder);
        quotient = (quotient << 1) | (fits ? 1 : 0);
        remainder = fits ? sub : remainder;
    }

    return negative ? -(int16_t)quotient : (int16_t)quotient;
}

// ============================================================================
// BITWISE INTEGER SQRT - IEEE-754 FREE
// ============================================================================

__device__ uint32_t bitwise_sqrt_u32(uint32_t radicand) {
    uint32_t bit = 1u << 15;  // Start from middle bit for 16-bit result
    uint32_t result = 0;

    #pragma unroll
    for (int i = 0; i < 16; i++) {
        uint32_t guess = result | bit;
        if ((uint64_t)guess * (uint64_t)guess <= (uint64_t)radicand) {
            result = guess;
        }
        bit >>= 1;
    }

    return result;
}

// ============================================================================
// NEWTON-RAPHSON DIVISION - Fast GPU Division
// ============================================================================

__device__ int32_t newton_raphson_div(int32_t numerator, int32_t denominator) {
    // Newton-Raphson reciprocal division: a/b = a * (1/b)
    // Step 1: Get absolute value for LUT lookup
    int32_t abs_denom = (denominator < 0) ? -denominator : denominator;

    // Step 2: LUT lookup - index with bits[13:6] (top 8 bits after sign+norm)
    // Single load from constant memory (fast, cached across warp)
    int index = (abs_denom >> 6) & 0xFF;
    int32_t recip = RECIPROCAL_LUT_16[index];

    // Step 3: Newton-Raphson refinement - 2 iterations
    // Formula: y_new = y * (2 - b*y)
    // Both denom and recip in 2^14 scale
    #pragma unroll
    for (int iter = 0; iter < 2; iter++) {
        int32_t prod = (abs_denom * recip) >> 14;  // 2^14 scale
        int32_t error = 0x8000 - prod;              // 2.0 - b*y in 2^14 scale
        recip = ((int64_t)recip * (int64_t)error) >> 14;
    }

    // Step 4: Multiply numerator by reciprocal
    // Both in 2^14 scale, product in 2^28, shift by 12 to get result
    int32_t sign_adjust = ((numerator < 0) != (denominator < 0)) ? -1 : 1;
    int32_t quotient = ((int64_t)numerator * (int64_t)recip * sign_adjust) >> 12;

    return quotient;
}

// ============================================================================
// SPIRIX SCALAR DIVISION
// ============================================================================

__device__ ScalarF4E4 spirix_div(ScalarF4E4 dividend, ScalarF4E4 divisor) {
    // Handle zero divisor
    if (divisor.fraction == 0) {
        if (dividend.fraction == 0) {
            // 0/0 = undefined
            return ScalarF4E4{UNDEFINED_PREFIX, AMBIGUOUS_EXPONENT};
        }
        // x/0 = infinity
        return ScalarF4E4{0x7FFF, 127};
    }

    // Handle zero dividend
    if (dividend.fraction == 0) {
        return ScalarF4E4{0, 0};
    }

    // Both normal values - perform division
    int32_t numerator = (int32_t)dividend.fraction;
    int32_t denominator = (int32_t)divisor.fraction;

    // Newton-Raphson division - Fast LUT + 2 iterations!
    int32_t quotient = newton_raphson_div(numerator, denominator);

    // Normalize: shift = max(leading_ones, leading_zeros) - 1
    int shift;
    if (quotient == 0) {
        return ScalarF4E4{0, 0};
    } else if (quotient < 0) {
        // Negative: count leading ones
        uint32_t uquot = (uint32_t)quotient;
        shift = __clz(~uquot) - 1;  // leading ones = clz(~x)
    } else {
        // Positive: count leading zeros
        shift = __clz((uint32_t)quotient) - 1;
    }

    // Shift quotient to normalize
    quotient <<= shift;
    int16_t fraction = (int16_t)(quotient >> 16);

    // Exponent adjustment: shift - (FRACTION_BITS - 1) = shift - 15
    int16_t expo_adjust = shift - 15;
    int16_t exponent = dividend.exponent - divisor.exponent - expo_adjust;

    return ScalarF4E4{fraction, exponent};
}

// ============================================================================
// SPIRIX SCALAR SQRT
// ============================================================================

__device__ ScalarF4E4 spirix_sqrt(ScalarF4E4 input) {
    // Handle edge cases
    if (input.fraction == 0) return ScalarF4E4{0, 0};
    if (input.fraction < 0) return ScalarF4E4{UNDEFINED_PREFIX, AMBIGUOUS_EXPONENT};

    // Exponent calculation
    int16_t exponent = input.exponent / 2;
    int16_t even = input.exponent & 1;
    exponent = exponent + even;
    if (input.exponent < 0 && even != 0) {
        exponent = exponent - 1;
    }

    // Sqrt the fraction
    uint32_t value = (uint32_t)(uint16_t)input.fraction;
    uint32_t x = value << (17 - even);
    uint32_t y = bitwise_sqrt_u32(x);

    // Normalization
    uint32_t leading_zeros = __clz(y) - 1;
    y = y << leading_zeros;
    int16_t shift_adjust = (int16_t)leading_zeros - 15;
    exponent = exponent + shift_adjust;

    int16_t fraction = (int16_t)(y >> 16);
    return ScalarF4E4{fraction, exponent};
}

#endif // SPIRIX_OPS_HIP
