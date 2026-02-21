/// Square root lookup table for 8-bit approximations
///
/// For a given 8-bit value, provides sqrt of that value shifted left by 8 bits
/// This gives a good initial guess for Newton-Raphson on larger bit widths

const fn generate_sqrt_lut() -> [u8; 256] {
    let mut lut = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        // We want sqrt((i << 8) | 0xFF) to get the high estimate
        // Simplified: sqrt of value in range [i.00, i.FF]
        let x = ((i as u32) << 8) | 0xFF;
        let mut y = 1u32;
        let mut y_squared = 1u32;

        // Find largest y where y² ≤ x
        while y_squared <= x {
            y += 1;
            y_squared = y * y;
        }
        y -= 1; // Back up one

        lut[i] = y as u8;
        i += 1;
    }
    lut
}

/// Lookup table mapping 8-bit input to approximate sqrt result
/// LUT[i] ≈ sqrt((i << 8) | 0xFF)
pub const SQRT_LUT: [u8; 256] = generate_sqrt_lut();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_lut_sanity() {
        // Test some known values
        assert_eq!(SQRT_LUT[0], 15); // sqrt(0x00FF) ≈ 15
        assert_eq!(SQRT_LUT[1], 22); // sqrt(0x01FF) ≈ 22
        assert_eq!(SQRT_LUT[255], 255); // sqrt(0xFFFF) = 255
    }

    #[test]
    fn test_sqrt_lut_monotonic() {
        // LUT should be monotonically increasing
        for i in 0..255 {
            assert!(
                SQRT_LUT[i] <= SQRT_LUT[i + 1],
                "LUT not monotonic at index {}: {} > {}",
                i,
                SQRT_LUT[i],
                SQRT_LUT[i + 1]
            );
        }
    }

    #[test]
    fn test_sqrt_lut_accuracy() {
        // Check a few specific cases
        for i in 0u8..=255 {
            let x = ((i as u32) << 8) | 0xFF;
            let lut_result = SQRT_LUT[i as usize] as u32;
            let actual_sqrt = (x as f64).sqrt() as u32;

            // LUT result should be within 1 of actual
            let diff = if lut_result > actual_sqrt {
                lut_result - actual_sqrt
            } else {
                actual_sqrt - lut_result
            };

            assert!(
                diff <= 1,
                "LUT[{}] = {}, but sqrt({}) ≈ {}, diff = {}",
                i,
                lut_result,
                x,
                actual_sqrt,
                diff
            );
        }
    }
}
