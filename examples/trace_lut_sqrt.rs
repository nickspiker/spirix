use spirix::ScalarF4E4;

fn main() {
    let input = ScalarF4E4::from(9u8);
    println!("Testing sqrt(9):");
    println!(
        "Input: fraction={}, exponent={}",
        input.fraction, input.exponent
    );

    // Manually trace through the algorithm
    let fraction = input.fraction as u32;
    let exponent = input.exponent;
    let even = (exponent & 1) as usize;

    println!("\nSetup:");
    println!("  fraction = 0x{:04x} = {}", fraction, fraction);
    println!("  exponent = {}", exponent);
    println!("  even = {}", even);

    let x = fraction << 17usize.wrapping_sub(even);
    println!("  x (radicand) = 0x{:08x} = {}", x, x);
    println!("  sqrt(x) should be ≈ {}", (x as f64).sqrt());

    // LUT seeding
    let lut_index = ((fraction >> 8) & 0xFF) as usize;
    println!("\nLUT seeding:");
    println!(
        "  lut_index = (0x{:04x} >> 8) & 0xFF = 0x{:02x} = {}",
        fraction, lut_index, lut_index
    );

    // Check what SQRT_LUT contains
    let lut_input = ((lut_index as u32) << 8) | 0xFF;
    println!(
        "  LUT models sqrt(0x{:04x}) = sqrt({})",
        lut_input, lut_input
    );
    println!("  Expected LUT value ≈ {}", (lut_input as f64).sqrt());

    let mut y = 136u32 << 8; // Hardcode the expected LUT value for now
    println!("  y (initial guess) = 0x{:08x} = {}", y, y);
    println!("  y should be ≈ sqrt(x) = {}", (x as f64).sqrt());
    println!("  Ratio: actual/guess = {}", (x as f64).sqrt() / y as f64);

    // Newton-Raphson iterations
    println!("\nNewton-Raphson iterations:");
    println!("  Loop condition: y <= x? {} <= {}? {}", y, x, y <= x);

    let mut iter = 0;
    while y <= x && iter < 10 {
        let new_y = (y + (x / y)) >> 1;
        println!(
            "  Iteration {}: y={}, x/y={}, new_y={}",
            iter,
            y,
            x / y,
            new_y
        );
        if new_y >= y {
            println!("    Converged (new_y >= y)");
            break;
        }
        y = new_y;
        iter += 1;
    }

    if y > x {
        println!("  Loop didn't run (y > x initially)");
    }

    println!("\nAfter Newton-Raphson:");
    println!("  y = 0x{:08x} = {}", y, y);

    // Normalization
    let leading_zeros = y.leading_zeros().wrapping_sub(1);
    let y_norm = y << leading_zeros;
    let shift_adjust = (leading_zeros as isize).wrapping_sub(15);

    println!("\nNormalization:");
    println!("  leading_zeros = {}", leading_zeros);
    println!("  y_norm = 0x{:08x}", y_norm);
    println!("  shift_adjust = {}", shift_adjust);

    let final_fraction = (y_norm >> 16) as i16;
    println!(
        "  final_fraction = 0x{:04x} = {}",
        final_fraction, final_fraction
    );

    let result = input.sqrt();
    println!("\nActual sqrt() result:");
    println!(
        "  fraction = 0x{:04x} = {}",
        result.fraction, result.fraction
    );
    println!("  exponent = {}", result.exponent);
    println!(
        "  as f64 ≈ {}",
        f64::from(result.fraction) / 65536.0 * 2f64.powi(result.exponent as i32)
    );
}
