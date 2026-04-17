use spirix::*;

macro_rules! show {
    ($label:expr, $s:expr) => {{
        let s = $s;
        let class = if s.is_undefined() {
            "\x1b[1;31m[℘]\x1b[0m"
        } else if s.is_zero() {
            "\x1b[1;33m[0]\x1b[0m"
        } else if s.is_infinite() {
            "\x1b[1;35m[∞]\x1b[0m"
        } else if s.exploded() {
            "\x1b[1;31m[↑]\x1b[0m"
        } else if s.vanished() {
            "\x1b[1;34m[↓]\x1b[0m"
        } else if s.is_normal() {
            "\x1b[1;32m[#]\x1b[0m"
        } else {
            "\x1b[1;37m[?]\x1b[0m"
        };
        println!("  {:<24} {} {:?}", $label, class, s);
    }};
}

fn main() {
    type S3 = ScalarF3E3;
    type S4 = ScalarF4E3;
    type S5 = ScalarF5E3;

    println!("\n\x1b[1;36m=== F3E3 (Scalar<i8, i8>) Constants ===\x1b[0m");
    show!("ZERO", S3::ZERO);
    show!("ONE", S3::ONE);
    show!("NEG_ONE", S3::NEG_ONE);
    show!("TWO", S3::TWO);
    show!("HALF", S3::HALF);
    show!("MAX", S3::MAX);
    show!("MIN", S3::MIN);
    show!("MIN_POS", S3::MIN_POS);
    show!("MAX_NEG", S3::MAX_NEG);
    show!("INFINITY", S3::INFINITY);
    show!("EXPLODED_POS", S3::EXPLODED_POS);
    show!("EXPLODED_NEG", S3::EXPLODED_NEG);
    show!("VANISHED_POS", S3::VANISHED_POS);
    show!("VANISHED_NEG", S3::VANISHED_NEG);
    show!("PI", S3::PI);
    show!("E", S3::E);

    println!("\n\x1b[1;36m=== F3E3 From<integer> ===\x1b[0m");
    show!("from(0)", S3::from(0));
    show!("from(1)", S3::from(1));
    show!("from(-1)", S3::from(-1));
    show!("from(2)", S3::from(2));
    show!("from(-2)", S3::from(-2));
    show!("from(7)", S3::from(7));
    show!("from(-7)", S3::from(-7));
    show!("from(42)", S3::from(42));
    show!("from(127)", S3::from(127));
    show!("from(-128)", S3::from(-128));

    println!("\n\x1b[1;36m=== F3E3 From<f64> ===\x1b[0m");
    show!("from(0.0)", S3::from(0.0));
    show!("from(-0.0)", S3::from(-0.0_f64));
    show!("from(1.0)", S3::from(1.0));
    show!("from(-1.0)", S3::from(-1.0));
    show!("from(0.5)", S3::from(0.5));
    show!("from(3.14159)", S3::from(3.14159));
    show!("from(f64::INFINITY)", S3::from(f64::INFINITY));
    show!("from(f64::NEG_INFINITY)", S3::from(f64::NEG_INFINITY));
    show!("from(f64::NAN)", S3::from(f64::NAN));

    println!("\n\x1b[1;36m=== F3E3 Arithmetic ===\x1b[0m");
    show!("1 + 1", S3::from(1) + S3::from(1));
    show!("7 - 7", S3::from(7) - S3::from(7));
    show!("3 * 5", S3::from(3) * S3::from(5));
    show!("15 / 3", S3::from(15) / S3::from(3));
    show!("7 % 3", S3::from(7) % S3::from(3));
    show!("-7 % 3", S3::from(-7) % S3::from(3));
    show!("1 / 0", S3::from(1) / S3::from(0));
    show!("0 / 0", S3::from(0) / S3::from(0));

    println!("\n\x1b[1;36m=== F4E3 (Scalar<i16, i8>) Spot Check ===\x1b[0m");
    show!("ZERO", S4::ZERO);
    show!("ONE", S4::ONE);
    show!("PI", S4::PI);
    show!("from(7)", S4::from(7));
    show!("from(1000)", S4::from(1000));

    println!("\n\x1b[1;36m=== F5E3 (Scalar<i32, i8>) Spot Check ===\x1b[0m");
    show!("ZERO", S5::ZERO);
    show!("ONE", S5::ONE);
    show!("PI", S5::PI);
    show!("from(7)", S5::from(7));
    show!("from(1000000)", S5::from(1000000));
}
