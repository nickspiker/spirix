use spirix::*;

fn main() {
    // Let's check what the actual constants are
    println!("ScalarF3E3 type info:");

    // Test a simple case first
    let zero = ScalarF3E3::ZERO;
    println!("ZERO.is_contiguous(): {}", zero.is_contiguous());

    let one = ScalarF3E3::ONE;
    println!("ONE.is_contiguous(): {}", one.is_contiguous());

    // Now try 5 in parts
    println!("\nTesting value 5...");
    let val = ScalarF3E3::from(5i64);
    println!("Value: {}", val.to_f64());
    println!("Exponent: {}", val.exponent);
    println!("Fraction: {:08b}", val.fraction);

    // Try to call is_contiguous with better error handling
    std::panic::set_hook(Box::new(|panic_info| {
        println!("PANIC: {}", panic_info);
    }));

    let result = std::panic::catch_unwind(|| {
        val.is_contiguous()
    });

    match result {
        Ok(is_cont) => println!("is_contiguous(): {}", is_cont),
        Err(_) => println!("is_contiguous() panicked!"),
    }
}