use approx::assert_relative_eq;
use spirix::*;
use std::f32::consts::PI;

// Macro to test basic complex arithmetic for all circle types
macro_rules! test_complex_for_type {
    ($circle_type:ident) => {
        // Test exact complex arithmetic
        let z1 = $circle_type::from((3.0, 4.0)); // 3 + 4i
        let z2 = $circle_type::from((1.0, 2.0)); // 1 + 2i

        // Addition: (3+4i) + (1+2i) = (4+6i)
        let sum = z1 + z2;
        assert_eq!(sum, $circle_type::from((4.0, 6.0)));

        // Subtraction: (3+4i) - (1+2i) = (2+2i)
        let diff = z1 - z2;
        assert_eq!(diff, $circle_type::from((2.0, 2.0)));

        // Test with real numbers
        let real_add = z1 + $circle_type::from(5);
        assert_eq!(real_add, $circle_type::from((8.0, 4.0)));

        // Test constants
        let zero = $circle_type::ZERO;
        let one = $circle_type::ONE;
        let pos_i = $circle_type::POS_I;
        let neg_i = $circle_type::NEG_I;

        assert!(zero.is_zero());
        assert!(one.is_normal());
        assert!(pos_i.is_normal());
        assert!(neg_i.is_normal());

        // Test constant values
        assert_eq!(one.r(), $circle_type::from(1).r());
        assert_eq!(one.i(), $circle_type::ZERO.r());
        assert_eq!(pos_i.r(), $circle_type::ZERO.r());
        assert_eq!(pos_i.i(), $circle_type::from(1).r());
        assert_eq!(neg_i.r(), $circle_type::ZERO.r());
        assert_eq!(neg_i.i(), $circle_type::from(-1).r());

        // Test magnitude - |3+4i| = 5
        let z_345 = $circle_type::from((3.0, 4.0));
        let mag = z_345.magnitude();
        assert_eq!(mag, $circle_type::from(5).r());

        // Test magnitude squared - |3+4i|² = 25
        let mag_sq = z_345.magnitude_squared();
        assert_eq!(mag_sq, $circle_type::from(25).r());

        // Test conjugate - conj(3+4i) = 3-4i
        let conj = z_345.conjugate();
        assert_eq!(conj.r(), z_345.r());
        assert_eq!(conj.i(), -z_345.i());

        // Test zero operations
        assert_eq!(z1 + zero, z1);
        assert_eq!(zero + z1, z1);
        assert_eq!(z1 * zero, zero);
        assert_eq!(zero * z1, zero);

        // Test identity
        assert_eq!(z1 * one, z1);
        assert_eq!(one * z1, z1);

        // Test imaginary unit property: i² = -1
        let i_squared = pos_i * pos_i;
        assert_eq!(i_squared, -one);

        // Test negation
        let neg_z1 = -z1;
        assert_eq!(neg_z1.r(), -z1.r());
        assert_eq!(neg_z1.i(), -z1.i());

        // Test state checks
        assert!(z1.is_normal());
        assert!(zero.is_zero());
        assert!(!z1.is_zero());
        assert!(!zero.is_normal());

        // Test undefined propagation
        let undefined = $circle_type::from(1) / zero;
        assert!(undefined.is_undefined());
        let propagated = undefined + z1;
        assert!(propagated.is_undefined());
    };
}

// Macro to generate test functions for all circle types
macro_rules! test_all_circle_types {
    ($($circle_type:ident),+) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_ $circle_type:lower _comprehensive>]() {
                    test_complex_for_type!($circle_type);
                }
            }
        )+
    };
}

// Generate comprehensive tests for all 25 circle types
test_all_circle_types!(
    CircleF3E3, CircleF3E4, CircleF3E5, CircleF3E6, CircleF3E7, CircleF4E3, CircleF4E4, CircleF4E5,
    CircleF4E6, CircleF4E7, CircleF5E3, CircleF5E4, CircleF5E5, CircleF5E6, CircleF5E7, CircleF6E3,
    CircleF6E4, CircleF6E5, CircleF6E6, CircleF6E7, CircleF7E3, CircleF7E4, CircleF7E5, CircleF7E6,
    CircleF7E7
);

#[cfg(test)]
mod basic_operations {
    use super::*;

    #[test]
    fn test_circle_creation() {
        // Create from tuple
        let z1 = CircleF5E3::from((3.0, 4.0));
        assert!(z1.is_normal());

        // Create from scalar (real number)
        let z2 = CircleF5E3::from(5);
        assert!(z2.is_normal());

        // Create from single scalar
        let z3 = CircleF5E3::from(ScalarF5E3::from(7));
        assert!(z3.is_normal());

        // Extract components
        let real = z1.r();
        let imag = z1.i();

        let real_f32: f32 = real.into();
        let imag_f32: f32 = imag.into();

        assert_relative_eq!(real_f32, 3.0, epsilon = 1e-5);
        assert_relative_eq!(imag_f32, 4.0, epsilon = 1e-5);
    }

    #[test]
    fn test_circle_constants() {
        // Test important constants
        let zero = CircleF5E3::ZERO;
        let one = CircleF5E3::ONE;
        let pos_i = CircleF5E3::POS_I;
        let neg_i = CircleF5E3::NEG_I;

        assert!(zero.is_zero());
        assert!(one.is_normal());
        assert!(pos_i.is_normal());
        assert!(neg_i.is_normal());

        // Check values
        let one_real: f32 = one.r().into();
        let one_imag: f32 = one.i().into();
        assert_relative_eq!(one_real, 1.0, epsilon = 1e-6);
        assert_relative_eq!(one_imag, 0.0, epsilon = 1e-6);

        let pos_i_real: f32 = pos_i.r().into();
        let pos_i_imag: f32 = pos_i.i().into();
        assert_relative_eq!(pos_i_real, 0.0, epsilon = 1e-6);
        assert_relative_eq!(pos_i_imag, 1.0, epsilon = 1e-6);

        let neg_i_real: f32 = neg_i.r().into();
        let neg_i_imag: f32 = neg_i.i().into();
        assert_relative_eq!(neg_i_real, 0.0, epsilon = 1e-6);
        assert_relative_eq!(neg_i_imag, -1.0, epsilon = 1e-6);
    }

    #[test]
    fn test_complex_arithmetic() {
        let z1 = CircleF5E3::from((3.0, 4.0)); // 3 + 4i
        let z2 = CircleF5E3::from((1.0, -2.0)); // 1 - 2i

        // Addition: (3+4i) + (1-2i) = (4+2i)
        let sum = z1 + z2;
        assert!(sum.is_normal());
        let sum_real: f32 = sum.r().into();
        let sum_imag: f32 = sum.i().into();
        assert_relative_eq!(sum_real, 4.0, epsilon = 1e-5);
        assert_relative_eq!(sum_imag, 2.0, epsilon = 1e-5);

        // Subtraction: (3+4i) - (1-2i) = (2+6i)
        let diff = z1 - z2;
        assert!(diff.is_normal());
        let diff_real: f32 = diff.r().into();
        let diff_imag: f32 = diff.i().into();
        assert_relative_eq!(diff_real, 2.0, epsilon = 1e-5);
        assert_relative_eq!(diff_imag, 6.0, epsilon = 1e-5);

        // Multiplication: (3+4i) * (1-2i) = 3 - 6i + 4i - 8i^2 = 3 - 2i + 8 = 11 - 2i
        let product = z1 * z2;
        assert!(product.is_normal());
        let prod_real: f32 = product.r().into();
        let prod_imag: f32 = product.i().into();
        assert_relative_eq!(prod_real, 11.0, epsilon = 1e-4);
        assert_relative_eq!(prod_imag, -2.0, epsilon = 1e-4);

        // Division: (3+4i) / (1-2i) = (3+4i)(1+2i) / (1+4) = (3+6i+4i+8i^2)/5 = (3+10i-8)/5 = (-5+10i)/5 = -1+2i
        let quotient = z1 / z2;
        assert!(quotient.is_normal());
        let quot_real: f32 = quotient.r().into();
        let quot_imag: f32 = quotient.i().into();
        assert_relative_eq!(quot_real, -1.0, epsilon = 1e-4);
        assert_relative_eq!(quot_imag, 2.0, epsilon = 1e-4);
    }
}

#[cfg(test)]
mod complex_specific_operations {
    use super::*;

    #[test]
    fn test_conjugate() {
        let z = CircleF5E3::from((3.0, 4.0)); // 3 + 4i
        let conj = z.conjugate(); // 3 - 4i

        assert!(conj.is_normal());
        let conj_real: f32 = conj.r().into();
        let conj_imag: f32 = conj.i().into();
        assert_relative_eq!(conj_real, 3.0, epsilon = 1e-5);
        assert_relative_eq!(conj_imag, -4.0, epsilon = 1e-5);

        // Conjugate properties
        let z_conj_conj = conj.conjugate();
        assert_eq!(z_conj_conj, z); // conjugate of conjugate is original

        // z * conjugate(z) = |z|^2
        let product = z * conj;
        let prod_real: f32 = product.r().into();
        let prod_imag: f32 = product.i().into();
        assert_relative_eq!(prod_real, 25.0, epsilon = 1e-4); // 3^2 + 4^2 = 25
        assert_relative_eq!(prod_imag, 0.0, epsilon = 1e-5); // Should be purely real
    }

    #[test]
    fn test_magnitude() {
        let z = CircleF5E3::from((3.0, 4.0)); // 3 + 4i

        // |3 + 4i| = sqrt(3^2 + 4^2) = sqrt(25) = 5
        let mag = z.magnitude();
        assert!(mag.is_normal());
        let mag_f32: f32 = mag.into();
        assert_relative_eq!(mag_f32, 5.0, epsilon = 1e-5);

        // Magnitude squared is more efficient
        let mag_sq = z.magnitude_squared();
        assert!(mag_sq.is_normal());
        let mag_sq_f32: f32 = mag_sq.into();
        assert_relative_eq!(mag_sq_f32, 25.0, epsilon = 1e-5);

        // Test with pure real number
        let real = CircleF5E3::from((-7.0, 0.0));
        let real_mag = real.magnitude();
        let real_mag_f32: f32 = real_mag.into();
        assert_relative_eq!(real_mag_f32, 7.0, epsilon = 1e-5);

        // Test with pure imaginary number
        let imag = CircleF5E3::from((0.0, -5.0));
        let imag_mag = imag.magnitude();
        let imag_mag_f32: f32 = imag_mag.into();
        assert_relative_eq!(imag_mag_f32, 5.0, epsilon = 1e-5);
    }

    #[test]
    fn test_sign_unit_vector() {
        let z = CircleF5E3::from((3.0, 4.0)); // 3 + 4i, |z| = 5

        let unit = z.sign(); // Should be (3/5) + (4/5)i = 0.6 + 0.8i
        assert!(unit.is_normal());

        let unit_real: f32 = unit.r().into();
        let unit_imag: f32 = unit.i().into();
        assert_relative_eq!(unit_real, 0.6, epsilon = 1e-4);
        assert_relative_eq!(unit_imag, 0.8, epsilon = 1e-4);

        // Unit vector should have magnitude 1
        let unit_mag = unit.magnitude();
        let unit_mag_f32: f32 = unit_mag.into();
        assert_relative_eq!(unit_mag_f32, 1.0, epsilon = 1e-4);
    }
}

#[cfg(test)]
mod circle_scalar_interactions {
    use super::*;

    #[test]
    fn test_circle_scalar_arithmetic() {
        let z = CircleF5E3::from((3.0, 4.0)); // 3 + 4i
        let s = ScalarF5E3::from(2);

        // Circle + Scalar
        let sum = z + s; // (3+4i) + 2 = (5+4i)
        let sum_real: f32 = sum.r().into();
        let sum_imag: f32 = sum.i().into();
        assert_relative_eq!(sum_real, 5.0, epsilon = 1e-5);
        assert_relative_eq!(sum_imag, 4.0, epsilon = 1e-5);

        // Circle * Scalar
        let product = z * s; // (3+4i) * 2 = (6+8i)
        let prod_real: f32 = product.r().into();
        let prod_imag: f32 = product.i().into();
        assert_relative_eq!(prod_real, 6.0, epsilon = 1e-5);
        assert_relative_eq!(prod_imag, 8.0, epsilon = 1e-5);

        // Scalar + Circle (should be commutative)
        let sum2 = s + z;
        assert_eq!(sum, sum2);

        // Circle / Scalar
        let quotient = z / s; // (3+4i) / 2 = (1.5+2i)
        let quot_real: f32 = quotient.r().into();
        let quot_imag: f32 = quotient.i().into();
        assert_relative_eq!(quot_real, 1.5, epsilon = 1e-5);
        assert_relative_eq!(quot_imag, 2.0, epsilon = 1e-5);
    }

    #[test]
    fn test_circle_rust_primitive_ops() {
        let z = CircleF5E3::from((3.0, 4.0));

        // Operations with Rust primitives
        let sum: CircleF5E3 = z + 2.0; // Should work with f32/f64
        let sum_real: f32 = sum.r().into();
        let sum_imag: f32 = sum.i().into();
        assert_relative_eq!(sum_real, 5.0, epsilon = 1e-5);
        assert_relative_eq!(sum_imag, 4.0, epsilon = 1e-5);

        let product: CircleF5E3 = z * 0.5;
        let prod_real: f32 = product.r().into();
        let prod_imag: f32 = product.i().into();
        assert_relative_eq!(prod_real, 1.5, epsilon = 1e-5);
        assert_relative_eq!(prod_imag, 2.0, epsilon = 1e-5);
    }
}

#[cfg(test)]
mod special_circle_values {
    use super::*;

    #[test]
    fn test_circle_zero_operations() {
        let zero = CircleF5E3::ZERO;
        let z = CircleF5E3::from((3.0, 4.0));

        // Addition with zero
        let result = z + zero;
        assert_eq!(result, z);

        // Multiplication with zero
        let result2 = z * zero;
        assert!(result2.is_zero());

        // Division by zero should create undefined state
        let div_by_zero = z / zero;
        assert!(div_by_zero.is_undefined());
    }

    #[test]
    fn test_circle_undefined_states() {
        let zero = CircleF5E3::ZERO;
        let normal = CircleF5E3::from((1.0, 2.0));

        // Division by zero
        let div_zero = normal / zero;
        assert!(div_zero.is_undefined());

        // Undefined values should propagate
        let propagated = div_zero + normal;
        assert!(propagated.is_undefined());
    }
}

#[cfg(test)]
mod complex_mathematical_functions {
    use super::*;

    #[test]
    fn test_complex_powers() {
        let z = CircleF5E3::from((1.0, 1.0)); // 1 + i

        // Square
        let z_squared = z.square();
        // (1+i)^2 = 1 + 2i + i^2 = 1 + 2i - 1 = 2i
        let sq_real: f32 = z_squared.r().into();
        let sq_imag: f32 = z_squared.i().into();
        assert_relative_eq!(sq_real, 0.0, epsilon = 1e-4);
        assert_relative_eq!(sq_imag, 2.0, epsilon = 1e-4);

        // Square root
        let sqrt_result = z.sqrt();
        assert!(sqrt_result.is_normal());

        // Verify: (sqrt(z))^2 ≈ z
        let verification = sqrt_result.square();
        let ver_real: f32 = verification.r().into();
        let ver_imag: f32 = verification.i().into();
        let orig_real: f32 = z.r().into();
        let orig_imag: f32 = z.i().into();
        assert_relative_eq!(ver_real, orig_real, epsilon = 1e-3);
        assert_relative_eq!(ver_imag, orig_imag, epsilon = 1e-3);

        // Power with scalar exponent
        let cube = z.pow(ScalarF5E3::from(3));
        assert!(cube.is_normal());
    }

    #[test]
    fn test_complex_exponential() {
        let z = CircleF5E3::from((0.0, PI)); // πi

        // e^(πi) = -1 (Euler's identity)
        let exp_pi_i = z.exp();
        assert!(exp_pi_i.is_normal());

        let exp_real: f32 = exp_pi_i.r().into();
        let exp_imag: f32 = exp_pi_i.i().into();
        assert_relative_eq!(exp_real, -1.0, epsilon = 1e-3);
        assert_relative_eq!(exp_imag, 0.0, epsilon = 1e-3);

        // e^0 = 1
        let zero = CircleF5E3::ZERO;
        let exp_zero = zero.exp();
        let exp_zero_real: f32 = exp_zero.r().into();
        let exp_zero_imag: f32 = exp_zero.i().into();
        assert_relative_eq!(exp_zero_real, 1.0, epsilon = 1e-5);
        assert_relative_eq!(exp_zero_imag, 0.0, epsilon = 1e-5);
    }
}

#[cfg(test)]
mod modular_operations {
    use super::*;

    #[test]
    fn test_complex_modulo() {
        let z = CircleF5E3::from((7.0, 9.0));
        let modulus = CircleF5E3::from((3.0, 2.0));

        // Component-wise modulo
        let result = z.modulo(modulus);
        assert!(result.is_normal());

        // The result should have both components properly reduced
        let result_real: f32 = result.r().into();
        let result_imag: f32 = result.i().into();

        // 7 mod 3 = 1, 9 mod 2 = 1
        assert_relative_eq!(result_real, 1.0, epsilon = 1e-5);
        assert_relative_eq!(result_imag, 1.0, epsilon = 1e-5);

        // Regular modulo operator
        let result2 = z % modulus;
        assert_eq!(result, result2);
    }
}

#[cfg(test)]
mod random_testing {
    use super::*;

    #[test]
    fn test_complex_random_generation() {
        for _ in 0..10 {
            // Random circle should be inside unit circle
            let random_circle = CircleF5E3::random();
            let magnitude = random_circle.magnitude();
            let mag_f32: f32 = magnitude.into();

            assert!(random_circle.is_normal() || random_circle.is_zero());
            assert!(mag_f32 <= 1.0 + 1e-5); // Allow small epsilon for floating point

            // Gaussian random
            let random_gauss = CircleF5E3::random_gauss();
            assert!(
                random_gauss.is_normal()
                    || random_gauss.is_zero()
                    || random_gauss.exploded()
                    || random_gauss.vanished()
            );
        }
    }
}
