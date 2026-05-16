//! Quick sanity probe for Scalar::random() under N0 format.
use spirix::*;

fn main() {
    let mut normal = 0;
    let mut zero = 0;
    let mut vanished = 0;
    let mut exploded = 0;
    let mut undef = 0;
    let mut infinite = 0;

    for _ in 0..100_000 {
        let r: ScalarF3E3 = ScalarF3E3::random();
        if r.is_undefined() {
            undef += 1;
        } else if r.is_zero() {
            zero += 1;
        } else if r.is_infinite() {
            infinite += 1;
        } else if r.exploded() {
            exploded += 1;
        } else if r.vanished() {
            vanished += 1;
        } else {
            normal += 1;
        }
    }

    println!("F3E3 random 100k trials:");
    println!("  normal:   {normal}");
    println!("  zero:     {zero}");
    println!("  vanished: {vanished}");
    println!("  exploded: {exploded}");
    println!("  infinite: {infinite}");
    println!("  undef:    {undef}");

    // Show a few samples
    println!("\nSamples:");
    for _ in 0..5 {
        let r: ScalarF5E3 = ScalarF5E3::random();
        println!("  {r}");
    }

    // Gaussian sanity
    println!("\nGauss samples (F5E3):");
    let mut sum = ScalarF5E3::ZERO;
    let mut sumsq = ScalarF5E3::ZERO;
    let n = 10_000;
    for _ in 0..n {
        let g: ScalarF5E3 = ScalarF5E3::random_gauss();
        sum = sum + g;
        sumsq = sumsq + g * g;
    }
    let n_s: ScalarF5E3 = (n as i32).into();
    let mean = sum / n_s;
    let var = sumsq / n_s - mean * mean;
    println!("  n={n} mean={mean} var={var} (expect mean~0, var~1)");
}
