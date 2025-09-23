use spirix::*;

#[test]
fn test_reciprocal_real() {
    let real = CircleF5E3::from(4.0);
    let recip = real.reciprocal();
    let expected = CircleF5E3::ONE / real;
    assert_eq!(recip.r(), expected.r());
    assert_eq!(recip.i(), expected.i());
}

#[test] 
fn test_reciprocal_imaginary() {
    let imag = CircleF5E3::from((0.0, 2.0));
    let recip = imag.reciprocal();
    let expected = CircleF5E3::ONE / imag;
    assert_eq!(recip.r(), expected.r());
    assert_eq!(recip.i(), expected.i());
}

#[test]
fn test_reciprocal_complex() {
    let complex = CircleF5E3::from((3.0, 4.0));
    let recip = complex.reciprocal();
    let expected = CircleF5E3::ONE / complex;
    assert_eq!(recip.r(), expected.r());
    assert_eq!(recip.i(), expected.i());
}

#[test]
fn test_reciprocal_identity() {
    let z = CircleF5E3::from((1.0, 1.0));
    let recip_z = z.reciprocal();
    let product = z * recip_z;
    // Should be very close to 1
    assert!((product.r() - 1.0).magnitude() < 0.01);
    assert!(product.i().magnitude() < 0.01);
}
