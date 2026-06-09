use spirix::ScalarF4E4;

fn main() {
    test_value(9u8);
    test_value(25u8);
    test_value(4u8);
}

fn test_value(input_val: u8) {
    let input = ScalarF4E4::from(input_val);
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!(
        "║  Testing sqrt({:3})                                        ║",
        input_val
    );
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("\nInput:");
    println!("  Value: {}", input);
    println!("  Bits:  {:#?}", input);

    // Current implementation result
    let current_result = input.sqrt();
    println!("\nCurrent sqrt() result:");
    println!("  Value: {}", current_result);
    println!("  Bits:  {:#?}", current_result);
    let as_u8 = u8::try_from(current_result);
    println!("  as u8: {:?}", as_u8);

    // Manual non-restoring algorithm with FULL trace
    println!("\n--- Non-Restoring Algorithm Trace ---");
    let exponent_half = input.exponent / 2;
    let even = (input.exponent & 1) as usize;
    let mut exp_result = exponent_half + (even as i16);
    // Adjust for negative odd exponents
    if input.exponent < 0 && even != 0 {
        exp_result -= 1;
    }

    println!("Exponent calculation:");
    println!("  input.exponent = {}", input.exponent);
    println!("  exponent/2 = {}", exponent_half);
    println!("  even bit = {}", even);
    println!("  exp_result = {}", exp_result);

    let x = (input.fraction as u32) << (17 - even);
    println!("\nRadicand:");
    println!("  fraction = {}", input.fraction);
    println!("  shifted by {} bits: x = 0x{:08x}", 17 - even, x);

    let mut y = 0u32;
    let mut bit = 1u32 << 15;
    println!("\nIterations (all 16):");

    // Non-restoring: track remainder, NO MULTIPLICATION! Based on: (y + bit)² = y² + 2*y*bit + bit² delta = 2*y*bit + bit² = ((2*y) << bit_pos) + (bit << bit_pos) = (2*y + bit) << bit_pos
    let mut remainder = x as u64;
    let mut y_squared: u64 = 0;
    let mut bit_pos = 15; // bit = 1 << bit_pos

    for iter in 0..16 {
        // delta = (2*y + bit) << bit_pos This is multiplication-free!
        let delta = (((y << 1) + bit) as u64) << bit_pos;

        print!(
            "  [{:2}] bit_pos={:2}, y={:5}, y²=0x{:016x}, delta=0x{:016x}, rem=0x{:016x}",
            iter, bit_pos, y, y_squared, delta, remainder
        );

        if remainder >= delta {
            remainder -= delta;
            y_squared += delta;
            y |= bit;
            println!(" -> YES: y={}", y);
        } else {
            println!(" -> NO");
        }
        bit >>= 1;
        bit_pos -= 1;
    }

    println!("\nAfter loop:");
    println!("  y = {} (0x{:04x})", y, y);

    // Normalization
    let leading_zeros = y.leading_zeros();
    let shift = if leading_zeros > 0 {
        leading_zeros - 1
    } else {
        0
    };
    let normalized = y << shift;
    let exp_adjust = (shift as i16) - 15;
    let final_exp = exp_result + exp_adjust;
    let final_frac = (normalized >> 16) as i16;

    println!("\nNormalization:");
    println!("  leading_zeros = {}", leading_zeros);
    println!("  shift = {}", shift);
    println!("  normalized = 0x{:08x}", normalized);
    println!("  exp_adjust = {}", exp_adjust);
    println!("  final_exp = {}", final_exp);
    println!("  final_frac = {}", final_frac);

    let manual_result = ScalarF4E4 {
        fraction: final_frac,
        exponent: final_exp,
    };

    println!("\nManual result:");
    println!("  Value: {}", manual_result);
    println!("  Bits:  {:#?}", manual_result);
    let manual_as_u8 = u8::try_from(manual_result);
    println!("  as u8: {:?}", manual_as_u8);

    println!("\n═══ COMPARISON ═══");
    println!("Current: {:?}", as_u8);
    println!("Manual:  {:?}", manual_as_u8);
    println!("Match: {}", as_u8 == manual_as_u8);
}
