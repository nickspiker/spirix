use spirix::*;

fn main() {
    // Test that our bitwise implementations work
    let a = ScalarF5E3::from(0b1010u8); // 10
    let b = 0b1100i16; // 12

    // Test Scalar & primitive
    let and_result = a & b;
    println!("a & b = {:?}", and_result);

    // Test primitive & Scalar
    let reverse_and = b & a;
    println!("b & a = {:?}", reverse_and);

    // Test equality
    let expected = ScalarF5E3::from(8);
    println!("and_result == expected: {}", and_result == expected);
    println!("reverse_and == expected: {}", reverse_and == expected);

    // Test OR
    let or_result = a | b;
    let expected_or = ScalarF5E3::from(14);
    println!("a | b = {:?}, expected = {:?}, equal: {}", or_result, expected_or, or_result == expected_or);

    // Test XOR
    let xor_result = a ^ b;
    let expected_xor = ScalarF5E3::from(6);
    println!("a ^ b = {:?}, expected = {:?}, equal: {}", xor_result, expected_xor, xor_result == expected_xor);
}