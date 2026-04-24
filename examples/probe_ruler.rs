//! Dumb-dumb sanity test across every scalar op after the v0.1 ruler flip.
//! Just "do the basic expected values come back?"
use spirix::*;

fn main() {
    type S = ScalarF5E3;
    let one: S = 1.into();
    let two: S = 2.into();
    let three: S = 3.into();
    let half: S = 0.5.into();

    fn chk<T: Into<f64> + Copy>(label: &str, got: T, want: f64) {
        let g: f64 = got.into();
        let ok = (g - want).abs() < 0.01 || (want != 0.0 && ((g - want) / want).abs() < 0.01);
        let mark = if ok { "✓" } else { "✗" };
        println!("  {mark} {label:<28} got={g} expect={want}");
    }

    println!("=== Basic arithmetic ===");
    chk("1 + 1", one + one, 2.0);
    chk("2 + 2", two + two, 4.0);
    chk("3 - 1", three - one, 2.0);
    chk("2 - 3", two - three, -1.0);
    chk("1 * 1", one * one, 1.0);
    chk("2 * 1", two * one, 2.0);
    chk("2 * 3", two * three, 6.0);
    chk("3 / 2", three / two, 1.5);
    chk("1 / 2", one / two, 0.5);
    chk("1 / 1", one / one, 1.0);

    println!("\n=== Negation/unary ===");
    chk("-1", -one, -1.0);
    chk("-(-1)", -(-one), 1.0);
    chk("abs(-3)", (-three).magnitude(), 3.0);

    println!("\n=== Comparison (returns bool, print inline) ===");
    println!("  1 < 2: {} (expect true)", one < two);
    println!("  2 < 1: {} (expect false)", two < one);
    println!("  2 == 2: {} (expect true)", two == two);

    println!("\n=== Sqrt / square ===");
    chk("sqrt(4)", S::from(4).sqrt(), 2.0);
    chk("sqrt(2)", two.sqrt(), 1.41421356);
    chk("sqrt(1)", one.sqrt(), 1.0);
    chk("square(3)", three.square(), 9.0);

    println!("\n=== Log / exp ===");
    chk("exp(0)", S::ZERO.exp(), 1.0);
    chk("exp(1)", one.exp(), 2.71828);
    chk("ln(1)", one.ln(), 0.0);
    chk("ln(e)", S::E.ln(), 1.0);
    chk("lb(2)", two.lb(), 1.0);
    chk("lb(8)", S::from(8).lb(), 3.0);

    println!("\n=== Trig ===");
    chk("sin(0)", S::ZERO.sin(), 0.0);
    chk("cos(0)", S::ZERO.cos(), 1.0);
    chk("sin(pi/2)", (S::PI / two).sin(), 1.0);
    chk("cos(pi)", S::PI.cos(), -1.0);
    chk("tan(0)", S::ZERO.tan(), 0.0);

    println!("\n=== Rounding ===");
    chk("floor(1.7)", S::from(1.7).floor(), 1.0);
    chk("ceil(1.2)", S::from(1.2).ceil(), 2.0);
    chk("round(1.5)", S::from(1.5).round(), 2.0);

    println!("\n=== Modulus ===");
    chk("5 % 3", S::from(5) % three, 2.0);
    chk("10 % 4", S::from(10) % S::from(4), 2.0);

    println!("\n=== Bitwise (at F5E3, 32-bit frac) ===");
    chk("1.5 & 1.25", S::from(1.5) & S::from(1.25), 1.0);

    println!("\n=== Shift ===");
    chk("3 << 1", three << 1, 6.0);
    chk("8 >> 2", S::from(8) >> 2, 2.0);

    println!("\n=== Constants ===");
    chk("ONE", S::ONE, 1.0);
    chk("TWO", S::TWO, 2.0);
    chk("HALF", S::HALF, 0.5);
    chk("PI", S::PI, std::f64::consts::PI);
    chk("E", S::E, std::f64::consts::E);
    chk("SQRT_TWO", S::SQRT_TWO, std::f64::consts::SQRT_2);
    chk("LN_TWO", S::LN_TWO, std::f64::consts::LN_2);
    chk("HALF_PI", S::HALF_PI, std::f64::consts::FRAC_PI_2);
    chk("NEG_ONE", S::NEG_ONE, -1.0);
}
