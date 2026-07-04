use spirix::*;
fn main() {
    let base = ScalarF5E3::from(0.1);
    let exp = ScalarF5E3::from(100);
    let result = base.pow(exp);
    eprintln!("0.1^100 = {}", result.to_f64());
    eprintln!(
        "  frac=0x{:x} exp=0x{:x}",
        result.fraction as u32, result.exponent as u8
    );
    eprintln!(
        "  vanished={} exploded={} is_zero={}",
        result.vanished(),
        result.exploded(),
        result.is_zero()
    );

    let small = ScalarF5E3::from(1e-30);
    eprintln!("1e-30 = {}", small.to_f64());
    eprintln!(
        "  frac=0x{:x} exp=0x{:x}",
        small.fraction as u32, small.exponent as u8
    );
    eprintln!(
        "  vanished={} exploded={} is_zero={}",
        small.vanished(),
        small.exploded(),
        small.is_zero()
    );
}
