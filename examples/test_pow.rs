use spirix::*;
fn main() {
    let e = ScalarF5E3::from(100);
    let shifted: ScalarF5E3 = e >> 1u8;
    let floored = shifted.floor();
    eprintln!(
        "e=100, shifted={}, floored={}",
        shifted.to_f64(),
        floored.to_f64()
    );
    eprintln!(
        "shifted: frac=0x{:x} exp=0x{:x}",
        shifted.fraction as u32, shifted.exponent as u8
    );
    eprintln!(
        "floored: frac=0x{:x} exp=0x{:x}",
        floored.fraction as u32, floored.exponent as u8
    );
}
