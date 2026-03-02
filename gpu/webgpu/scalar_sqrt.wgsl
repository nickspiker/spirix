// ScalarF4E4 Square Root - WebGPU Compute Shader
// Direct port from HIP kernel with bitwise sqrt

// Constants
const AMBIGUOUS_EXPONENT: i32 = -32768;

// Packed storage: fraction in low 16 bits, exponent in high 16 bits
@group(0) @binding(0) var<storage, read> a_packed: array<u32>;
@group(0) @binding(1) var<storage, read_write> result_packed: array<u32>;

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

// Bitwise Integer Square Root (IEEE-754 free)
// Computes integer square root using binary search with fully unrolled loop
// Returns 16-bit result from 32-bit input
fn bitwise_sqrt_u32(radicand: u32) -> u32 {
    var bit = 1u << 15;  // Start from middle bit for 16-bit result
    var result = 0u;

    // Fully unrolled 16-iteration binary search
    for (var i = 0; i < 16; i++) {
        let guess = result | bit;
        // Use branchless: result = (guess² <= radicand) ? guess : result
        if (u64(guess) * u64(guess) <= u64(radicand)) {
            result = guess;
        }
        bit = bit >> 1u;
    }

    return result;
}

@compute @workgroup_size(256)
fn scalar_sqrt(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = i32(global_id.x);
    let n = i32(arrayLength(&a_packed));

    // Branchless valid mask
    let mask_valid = -i32(idx < n);

    // Load input
    let a_frac = unpack_fraction(a_packed[idx]);
    let a_exp = unpack_exponent(a_packed[idx]);

    // Check edge cases
    let mask_zero = -i32(a_frac == 0);
    let mask_negative = -i32(a_frac < 0);

    // Exponent calculation: divide by 2, handle odd exponents
    // For odd negative exponents, we need floor division
    var exponent = a_exp / 2;
    let even = a_exp & 1;  // 1 if odd, 0 if even
    exponent = exponent + even;  // Adjust for odd exponents

    // Correct for negative odd exponents: floor(neg/2) needs -1
    let neg_odd_adjust = -i32((a_exp < 0) & (even != 0));
    exponent = exponent + neg_odd_adjust;

    // Prepare fraction for sqrt: shift to position
    // For even exponents: shift by 17 bits
    // For odd exponents: shift by 16 bits (17 - 1)
    let value = u32(u16(a_frac));  // Convert to unsigned
    let shifted_value = value << u32(17 - even);

    // Compute sqrt using bitwise method
    var sqrt_result = bitwise_sqrt_u32(shifted_value);

    // Normalize the result
    let leading_zeros = u32(clz32(sqrt_result)) - 1u;
    sqrt_result = sqrt_result << leading_zeros;
    let shift_adjust = i32(leading_zeros) - 15;
    exponent = exponent + shift_adjust;

    // Extract normalized fraction (top 16 bits)
    var result_frac = i32(sqrt_result >> 16u);
    var result_exp = exponent;

    // Final result selection
    result_frac = (mask_zero & 0) |            // Zero input → zero fraction
                  (mask_negative & 0) |         // Negative input → zero fraction (undefined)
                  (~(mask_zero | mask_negative) & result_frac);

    result_exp = (mask_zero & AMBIGUOUS_EXPONENT) |        // Zero input → ambiguous
                 (mask_negative & AMBIGUOUS_EXPONENT) |    // Negative input → ambiguous (undefined)
                 (~(mask_zero | mask_negative) & result_exp);

    // Write result (only valid threads) - mask with valid
    let final_frac = result_frac & mask_valid;
    let final_exp = result_exp & mask_valid;
    result_packed[idx] = pack_scalar(final_frac, final_exp);
}
