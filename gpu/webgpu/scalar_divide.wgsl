// ScalarF4E4 Division - WebGPU Compute Shader
// Direct port from HIP kernel with Newton-Raphson + LUT

// Constants
const AMBIGUOUS_EXPONENT: i32 = -32768;
const FRACTION_BITS: i32 = 16;

// Packed storage: fraction in low 16 bits, exponent in high 16 bits
@group(0) @binding(0) var<storage, read> a_packed: array<u32>;
@group(0) @binding(1) var<storage, read> b_packed: array<u32>;
@group(0) @binding(2) var<storage, read_write> result_packed: array<u32>;
@group(0) @binding(3) var<storage, read> reciprocal_lut: array<i32, 256>;

// Unpack helpers - extract signed 16-bit values from packed u32
fn unpack_fraction(packed: u32) -> i32 {
    let val = i32(packed & 0xFFFFu);
    // Sign-extend from 16 to 32 bits
    return (val << 16) >> 16;
}

fn unpack_exponent(packed: u32) -> i32 {
    let val = i32(packed >> 16u);
    // Sign-extend from 16 to 32 bits
    return (val << 16) >> 16;
}

// Pack helpers - combine fraction and exponent into u32
fn pack_scalar(frac: i32, exp: i32) -> u32 {
    let f = u32(frac) & 0xFFFFu;
    let e = u32(exp) & 0xFFFFu;
    return (e << 16u) | f;
}

// Count leading zeros - WGSL builtin (same as HIP __clz)
fn clz32(x: u32) -> i32 {
    return i32(countLeadingZeros(x));
}

// Newton-Raphson Division: a / b using LUT + 2 iterations
fn newton_raphson_divide(numerator: i32, denominator: i32) -> i32 {
    // Get absolute values for computation
    let abs_numer = select(numerator, -numerator, numerator < 0);
    let abs_denom = select(denominator, -denominator, denominator < 0);
    let result_negative = (numerator < 0) != (denominator < 0);

    // LUT lookup: index by bits [13:6] of normalized denominator
    let index = (abs_denom >> 6) & 0xFF;
    var recip = reciprocal_lut[index];  // 2^14 scale

    // Newton-Raphson refinement: 2 iterations
    // Formula: y_new = y * (2 - b*y)
    for (var iter = 0; iter < 2; iter++) {
        let prod = (abs_denom * recip) >> 14;  // b*y in 2^14 scale
        let error = 0x8000 - prod;              // (2.0 - b*y) in 2^14 scale
        recip = i32((i64(recip) * i64(error)) >> 14);
    }

    // Multiply numerator by reciprocal: a * (1/b)
    // Both in 2^14 scale, shift by 12 for final result
    let product = i64(abs_numer) * i64(recip);
    let quotient = i32(product >> 12);

    return select(quotient, -quotient, result_negative);
}

@compute @workgroup_size(256)
fn scalar_divide(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = i32(global_id.x);
    let n = i32(arrayLength(&a_packed));

    // Branchless valid mask
    let mask_valid = -i32(idx < n);

    // Load inputs
    let a_frac = unpack_fraction(a_packed[idx]);
    let b_frac = unpack_fraction(b_packed[idx]);
    let a_exp = unpack_exponent(a_packed[idx]);
    let b_exp = unpack_exponent(b_packed[idx]);

    // Check edge cases
    let mask_div_by_zero = -i32(b_frac == 0);
    let mask_zero_dividend = -i32(a_frac == 0);

    // Newton-Raphson division (only if both non-zero)
    let quotient = newton_raphson_divide(a_frac, b_frac);

    // Count leading same bits for normalization
    let lz = clz32(u32(quotient));
    let lo = clz32(u32(~quotient));
    let leading = select(lo, lz, lz > lo);

    // Normalization shift (leading - 1 for N-1 position)
    let shift = leading - 1;

    // Normalize quotient and extract fraction
    let normalized = quotient << shift;
    var result_frac = normalized >> 16;

    // Exponent calculation: a_exp - b_exp - expo_adjust
    // expo_adjust = shift - 15 (accounts for normalization)
    let expo_adjust = shift - 15;
    let temp_exp = a_exp - b_exp - expo_adjust;

    // Final result selection
    result_frac = (mask_div_by_zero & 0x7FFF) |         // Infinity fraction
                  (mask_zero_dividend & 0) |             // Zero fraction
                  (~(mask_div_by_zero | mask_zero_dividend) & result_frac);

    let result_exp = (mask_div_by_zero & 127) |             // Infinity exponent (large)
                     (mask_zero_dividend & AMBIGUOUS_EXPONENT) |  // Zero exponent
                     (~(mask_div_by_zero | mask_zero_dividend) & temp_exp);

    // Write result (only valid threads) - mask with valid
    let final_frac = result_frac & mask_valid;
    let final_exp = result_exp & mask_valid;
    result_packed[idx] = pack_scalar(final_frac, final_exp);
}
