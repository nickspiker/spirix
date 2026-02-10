use spirix::ScalarF4E4;

#[test]
fn debug_scalar_representation() {
    for i in [0, 1, 2, 127, 128, 254, 255, 256] {
        let s = ScalarF4E4::from(i);
        println!(
            "Value {}: fraction={} (0x{:08x}), exponent={}",
            i, s.fraction, s.fraction as u32, s.exponent
        );

        // Show what the formula should give:
        let expected_value = if s.exponent >= 31 {
            "overflow"
        } else if s.exponent <= 0 {
            "underflow"
        } else {
            "normal"
        };
        println!("  State: {}", expected_value);
    }
}
