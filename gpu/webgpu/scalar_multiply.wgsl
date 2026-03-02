// ScalarF4E4 Multiplication - WebGPU Compute Shader
// Direct port from HIP kernel with identical logic

// Constants
const AMBIGUOUS_EXPONENT: i32 = -32768;
const FRACTION_BITS: i32 = 16;

// Packed storage: fraction in low 16 bits, exponent in high 16 bits
@group(0) @binding(0) var<storage, read> a_packed: array<u32>;
@group(0) @binding(1) var<storage, read> b_packed: array<u32>;
@group(0) @binding(2) var<storage, read_write> result_packed: array<u32>;

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

@compute @workgroup_size(256)
fn scalar_multiply(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = i32(global_id.x);
    let n = i32(arrayLength(&a_packed));

    // Branchless valid mask
    let mask_valid = -i32(idx < n);

    // Load inputs (i16 → i32 with sign extension)
    let a_frac = unpack_fraction(a_packed[idx]);
    let b_frac = unpack_fraction(b_packed[idx]);
    let a_exp = unpack_exponent(a_packed[idx]);
    let b_exp = unpack_exponent(b_packed[idx]);

    // Check for ambiguous inputs
    let mask_ambiguous = -i32((a_exp == AMBIGUOUS_EXPONENT) | (b_exp == AMBIGUOUS_EXPONENT));

    // Multiply fractions (i16 × i16 = i32 product)
    let product = a_frac * b_frac;

    // Check for zero
    let mask_zero = -i32(product == 0);

    // Count leading same bits
    let lz = clz32(u32(product));
    let lo = clz32(u32(~product));
    let leading = select(lo, lz, lz > lo);

    // Exponent adjustment: leading - 2 (accounts for double-width product)
    let expo_adjust = leading - 2;
    let shift = expo_adjust + 1;

    // Normalize: shift product and extract high half
    let normalized_product = product << shift;
    var result_frac = normalized_product >> FRACTION_BITS;

    // Add exponents and subtract adjustment
    let temp_exp = a_exp + b_exp - expo_adjust;

    // Check for overflow/underflow
    let mask_overflow = -i32(temp_exp > 32767);
    let mask_underflow = -i32(temp_exp <= AMBIGUOUS_EXPONENT);

    // Set exponent (overflow and underflow both → AMBIGUOUS)
    var result_exp = ((mask_overflow | mask_underflow) & AMBIGUOUS_EXPONENT) |
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

    // Write result (only valid threads) - mask with valid
    let final_frac = result_frac & mask_valid;
    let final_exp = result_exp & mask_valid;
    result_packed[idx] = pack_scalar(final_frac, final_exp);
}
