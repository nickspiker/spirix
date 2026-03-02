// ScalarF4E4 Addition - WebGPU Compute Shader
// Direct port from HIP kernel with identical logic

// Constants
const AMBIGUOUS_EXPONENT: i32 = -32768;
const FRACTION_BITS: i32 = 16;

// Packed storage: fraction in low 16 bits, exponent in high 16 bits
// This matches HIP's 4-byte struct layout exactly
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
fn scalar_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = i32(global_id.x);
    let n = i32(arrayLength(&a_packed));

    // Branchless valid mask (same as HIP: mask_valid = -(idx < n))
    let mask_valid = -i32(idx < n);

    // Load inputs (i16 → i32 with sign extension)
    let a_frac = unpack_fraction(a_packed[idx]);
    let b_frac = unpack_fraction(b_packed[idx]);
    let a_exp = unpack_exponent(a_packed[idx]);
    let b_exp = unpack_exponent(b_packed[idx]);

    // Check for ambiguous inputs
    let mask_ambiguous = -i32((a_exp == AMBIGUOUS_EXPONENT) | (b_exp == AMBIGUOUS_EXPONENT));

    // Exponent alignment
    let exp_diff = a_exp - b_exp;
    let abs_diff = select(exp_diff, -exp_diff, exp_diff < 0);
    let mask_a_larger = -i32(exp_diff >= 0);
    let mask_negligible = -i32(abs_diff >= FRACTION_BITS);

    // Fraction alignment and ADDITION
    let big_a = a_frac << abs_diff;
    let big_b = b_frac << abs_diff;

    var add_result = (mask_a_larger & (big_a + b_frac)) | (~mask_a_larger & (a_frac + big_b));
    var result_exp = (mask_a_larger & b_exp) | (~mask_a_larger & a_exp);

    // Sign-extend result from 16-bit to 32-bit
    add_result = (add_result << 16) >> 16;

    // Check for zero
    let mask_zero = -i32(add_result == 0);

    // Count leading same bits (handles overflow naturally)
    let lz = clz32(u32(add_result));
    let lo = clz32(u32(~add_result));
    let leading = select(lo, lz, lz > lo);

    // Calculate adjusted exponent
    let temp_exp = result_exp + 17 - leading;

    // Check for underflow
    let mask_underflow = -i32(temp_exp <= AMBIGUOUS_EXPONENT);

    // Normalize fraction (N-1 and N-2)
    let shift_n1 = leading - 1;
    let shift_n2 = leading - 2;
    let norm_n1 = (add_result << shift_n1) >> 16;
    let norm_n2 = (add_result << shift_n2) >> 16;

    var result_frac = (mask_underflow & norm_n2) | (~mask_underflow & norm_n1);
    result_exp = (mask_underflow & AMBIGUOUS_EXPONENT) | (~mask_underflow & temp_exp);

    // Final result selection (cascading priority)
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

    // Write result (only valid threads) - mask with valid
    let final_frac = result_frac & mask_valid;
    let final_exp = result_exp & mask_valid;
    result_packed[idx] = pack_scalar(final_frac, final_exp);
}
