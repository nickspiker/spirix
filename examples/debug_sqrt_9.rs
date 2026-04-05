use spirix::ScalarF4E4;

fn main() {
    let input = ScalarF4E4::from(9u8);

    println!("Testing sqrt(9) = 3");
    println!("==================\n");

    println!("Input:");
    println!("  fraction = 0x{:04x} = {}", input.fraction, input.fraction);
    println!("  exponent = {}\n", input.exponent);

    // Trace LUT-Newton
    let fraction = input.fraction as u32;
    let even = (input.exponent & 1) as usize;
    let x = fraction << (17 - even);

    println!("LUT-Newton trace:");
    println!("  x (radicand) = 0x{:08x}", x);

    let lut_index = ((fraction >> 8) & 0xFF) as usize;
    println!("  lut_index = 0x{:02x} = {}", lut_index, lut_index);

    // Simulate what the LUT should give
    let lut_approx = (((lut_index << 8) | 0xFF) as f64).sqrt() as u32;
    println!("  LUT[{}] ≈ {}", lut_index, lut_approx);

    let mut y = lut_approx << 8;
    println!("  Initial y = {} (0x{:08x})", y, y);
    println!("  Target ≈ {}", (x as f64).sqrt());

    // Newton-Raphson iterations
    println!("\n  Newton iterations:");
    let mut iter = 0;
    while y <= x && iter < 10 {
        let new_y = (y + (x / y)) >> 1;
        println!(
            "    [{}] y={} (0x{:x}), new_y={} (0x{:x}), diff={}",
            iter,
            y,
            y,
            new_y,
            new_y,
            new_y as i64 - y as i64
        );
        if new_y == y {
            println!("    Converged!");
            break;
        }
        y = new_y;
        iter += 1;
    }
    println!("  Final y = {} (0x{:08x})", y, y);
    println!("  Iterations: {}\n", iter);

    // Trace bitwise
    println!("Bitwise trace:");
    let radicand = fraction << (17 - even);
    println!("  radicand = 0x{:08x}", radicand);
    let mut bit = 1u32 << 15;
    let mut result = 0u32;
    let mut iterations = 0;

    while bit != 0 {
        let guess = result | bit;
        if (guess as u64) * (guess as u64) <= radicand as u64 {
            println!(
                "  [{}] bit=0x{:04x}, result=0x{:04x}, guess=0x{:04x}, guess²={} <= {} ✓",
                iterations,
                bit,
                result,
                guess,
                (guess as u64) * (guess as u64),
                radicand
            );
            result = guess;
            if (guess as u64) * (guess as u64) == radicand as u64 {
                println!("    Perfect square - early exit!");
                break;
            }
        }
        bit >>= 1;
        iterations += 1;
    }
    println!("  Final result = {} (0x{:08x})", result, result);
    println!(
        "  Iterations: {} (checked {} bits)\n",
        iterations, iterations
    );

    // Compare results
    let sqrt_result = input.sqrt();
    let newton_result = input.sqrt_newton();

    println!("Results:");
    println!(
        "  sqrt:        fraction=0x{:04x}, exponent={}",
        sqrt_result.fraction, sqrt_result.exponent
    );
    println!(
        "  sqrt_newton: fraction=0x{:04x}, exponent={}",
        newton_result.fraction, newton_result.exponent
    );
    println!(
        "  Match: {}",
        sqrt_result.fraction == newton_result.fraction
            && sqrt_result.exponent == newton_result.exponent
    );
}
