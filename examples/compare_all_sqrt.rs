use spirix::{ScalarF3E3, ScalarF4E4, ScalarF5E5, ScalarF6E6, ScalarF7E7};

fn test_8bit() {
    println!("═══ Testing 8-bit (ScalarF3E3) ═══");
    let mut mismatches = 0;
    let mut total = 0;

    for val in 0i8..=127 {
        let input = ScalarF3E3::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();
        total += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!(
                "  MISMATCH val={}: newton={:#?} bitwise={:#?}",
                val, newton, bitwise
            );
        }
    }

    println!(
        "  Total: {}, Mismatches: {}, Match rate: {:.2}%\n",
        total,
        mismatches,
        100.0 * (total - mismatches) as f64 / total as f64
    );
}

fn test_16bit() {
    println!("═══ Testing 16-bit (ScalarF4E4) ═══");
    let mut mismatches = 0;
    let mut total = 0;

    for val in 0u8..=255 {
        let input = ScalarF4E4::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();
        total += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!(
                "  MISMATCH val={}: newton={:#?} bitwise={:#?}",
                val, newton, bitwise
            );
        }
    }

    println!(
        "  Total: {}, Mismatches: {}, Match rate: {:.2}%\n",
        total,
        mismatches,
        100.0 * (total - mismatches) as f64 / total as f64
    );
}

fn test_32bit() {
    println!("═══ Testing 32-bit (ScalarF5E5) ═══");
    let mut mismatches = 0;
    let mut total = 0;

    // Test a representative sample
    for val in 0u16..=1000 {
        let input = ScalarF5E5::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();
        total += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!(
                "  MISMATCH val={}: newton={:#?} bitwise={:#?}",
                val, newton, bitwise
            );
        }
    }

    println!(
        "  Total: {}, Mismatches: {}, Match rate: {:.2}%\n",
        total,
        mismatches,
        100.0 * (total - mismatches) as f64 / total as f64
    );
}

fn test_64bit() {
    println!("═══ Testing 64-bit (ScalarF6E6) ═══");
    let mut mismatches = 0;
    let mut total = 0;

    // Test a representative sample
    for val in 0u16..=1000 {
        let input = ScalarF6E6::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();
        total += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!(
                "  MISMATCH val={}: newton={:#?} bitwise={:#?}",
                val, newton, bitwise
            );
        }
    }

    println!(
        "  Total: {}, Mismatches: {}, Match rate: {:.2}%\n",
        total,
        mismatches,
        100.0 * (total - mismatches) as f64 / total as f64
    );
}

fn test_128bit() {
    println!("═══ Testing 128-bit (ScalarF7E7) ═══");
    let mut mismatches = 0;
    let mut total = 0;

    // Test a representative sample
    for val in 0u16..=1000 {
        let input = ScalarF7E7::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();
        total += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!(
                "  MISMATCH val={}: newton={:#?} bitwise={:#?}",
                val, newton, bitwise
            );
        }
    }

    println!(
        "  Total: {}, Mismatches: {}, Match rate: {:.2}%\n",
        total,
        mismatches,
        100.0 * (total - mismatches) as f64 / total as f64
    );
    println!("  Note: 128-bit still uses multiplication (no U512 type available)");
}

fn main() {
    println!("Comparing sqrt() (Newton-Raphson) vs sqrt_bb() (bit-by-bit non-restoring)\n");

    test_8bit();
    test_16bit();
    test_32bit();
    test_64bit();
    test_128bit();

    println!("═══════════════════════════════════════");
    println!("All tests complete!");
}
