//! Quick sanity probes for trig, exponents, and statistics on N0 scalars.
//! Not exhaustive — just "do the basics produce sane results?"
use spirix::*;

fn p<T: std::fmt::Display>(label: &str, v: T) {
    println!("  {label:<28} = {v}");
}

fn main() {
    type S = ScalarF5E3;
    let pi: S = std::f64::consts::PI.into();
    let _half: S = 0.5.into();
    let one: S = 1.into();
    let two: S = 2.into();
    let e: S = std::f64::consts::E.into();
    let zero = S::ZERO;

    println!("=== Trigonometry (expected vs actual) ===");
    p("sin(0) expect 0", zero.sin());
    p("cos(0) expect 1", zero.cos());
    p("sin(pi) expect ~0", pi.sin());
    p("cos(pi) expect -1", pi.cos());
    p("sin(pi/2) expect 1", (pi / two).sin());
    p("cos(pi/2) expect ~0", (pi / two).cos());
    p("tan(0) expect 0", zero.tan());
    p("tan(pi/4) expect 1", (pi / ScalarF5E3::from(4)).tan());
    p("asin(1) expect pi/2", one.asin());
    p("acos(0) expect pi/2", zero.acos());
    p("atan(1) expect pi/4", one.atan());

    println!("\n=== Exponents / Logs ===");
    p("exp(0) expect 1", zero.exp());
    p("exp(1) expect e", one.exp());
    p("exp(2) expect e^2 ~7.389", two.exp());
    p("ln(1) expect 0", one.ln());
    p("ln(e) expect 1", e.ln());
    p("ln(2) expect 0.693", two.ln());
    p("lb(2) expect 1", two.lb());
    p("lb(8) expect 3", ScalarF5E3::from(8).lb());
    p("powb(0) expect 1", zero.powb());
    p("powb(3) expect 8", ScalarF5E3::from(3).powb());
    p("sqrt(4) expect 2", ScalarF5E3::from(4).sqrt());
    p("sqrt(2) expect 1.414", two.sqrt());
    p("square(3) expect 9", ScalarF5E3::from(3).square());

    println!("\n=== Round-trips ===");
    p("exp(ln(5)) expect 5", ScalarF5E3::from(5).ln().exp());
    p("sqrt(square(7)) expect 7", ScalarF5E3::from(7).square().sqrt());
    p("sin^2 + cos^2 @ 0.3", {
        let x: S = 0.3.into();
        let s = x.sin();
        let c = x.cos();
        s * s + c * c
    });

    println!("\n=== Statistics ===");
    let nums: [S; 5] = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    // mean requires slice or iterator API — just try manual
    let mut sum = S::ZERO;
    for n in &nums { sum = sum + *n; }
    let n5: S = 5.into();
    p("mean(1..5) expect 3", sum / n5);

    // basic_scalar's abs, sign, max, min, clamp
    let neg: S = (-3.5).into();
    p("abs(-3.5) expect 3.5", neg.magnitude());
    p("max(3,5) expect 5", S::from(3).max(S::from(5)));
    p("min(3,5) expect 3", S::from(3).min(S::from(5)));

    println!("\n=== Sigmoid ===");
    p("sigmoid(0) expect 0.5", zero.sigmoid());
    p("sigmoid(2) expect 0.881", two.sigmoid());
    p("sigmoid(-2) expect 0.119", (-two).sigmoid());
}
