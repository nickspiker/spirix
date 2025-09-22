use spirix::*;

fn main() {
    let a = ScalarF5E3::from(5u8);
    let b = 3i16;
    let c = ScalarF5E3::from(8f32);

    // Test min with mixed types - this should now work!
    let min_val = a.min(b);
    println!("a.min(b) = {:?} (expected: 3)", min_val);
    assert!(min_val == 3u32);

    // Test max with mixed types
    let max_val = a.max(c);
    println!("a.max(c) = {:?} (expected: 8)", max_val);
    assert!(max_val == 8i64);

    // Test clamp with mixed types
    let clamped = a.clamp(ScalarF5E3::from(b), c);
    println!("a.clamp(b, c) = {:?} (expected: 5)", clamped);
    assert!(clamped == 5f64);

    let clamped_low = ScalarF5E3::from(1usize).clamp(ScalarF5E3::from(b), c);
    println!("1.clamp(b, c) = {:?} (expected: 3)", clamped_low);
    assert!(clamped_low == 3isize);

    let clamped_high = ScalarF5E3::from(10u128).clamp(ScalarF5E3::from(b), c);
    println!("10.clamp(b, c) = {:?} (expected: 8)", clamped_high);
    assert!(clamped_high == 8);

    println!("All min/max/clamp operations work correctly with mixed types!");
}