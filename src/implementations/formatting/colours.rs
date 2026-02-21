/// A colour scheme defining RGB values for formatting output.
///
/// Used in debug formatting (`{:#?}`) to colourize the binary representation
/// based on the number's state (normal, exploded, vanished, etc.).
#[derive(Clone, Copy)]
pub struct ColourScheme {
    /// RGB colour values (red, green, blue) in the range 0-255
    pub colour: [u8; 3],
}

/// Colour palette for formatting Scalar and Circle values in debug mode.
///
/// Each field represents a different number state with its associated RGB colour.
/// These colours are applied when using alternate debug formatting (`{:#?}`).
///
/// ## Colour Meanings
///
/// - **Normal**: Standard finite numbers (positive=light red, negative=light blue)
/// - **Exploded**: Numbers too large to represent (positive=bright red, negative=bright blue)
/// - **Vanished**: Numbers too small to represent (positive=bright red, negative=bright blue)
/// - **Zero**: Exact zero (light green)
/// - **Undefined**: Invalid or undefined states (light magenta)
/// - **Integer exponent**: Positive exponent values (light yellow)
/// - **Fractional exponent**: Negative exponent values (light cyan)
#[derive(Clone, Copy)]
pub struct ScalarColours {
    /// Colour for normal positive numbers (light red: 0xff, 0x80, 0x80)
    pub normal_positive: ColourScheme,
    /// Colour for normal negative numbers (light blue: 0x80, 0x80, 0xff)
    pub normal_negative: ColourScheme,
    /// Colour for exploded positive numbers (bright red: 0xff, 0x00, 0x00)
    pub exploded_positive: ColourScheme,
    /// Colour for exploded negative numbers (bright blue: 0x00, 0x00, 0xff)
    pub exploded_negative: ColourScheme,
    /// Colour for vanished positive numbers (bright red: 0xff, 0x00, 0x00)
    pub vanished_positive: ColourScheme,
    /// Colour for vanished negative numbers (bright blue: 0x00, 0x00, 0xff)
    pub vanished_negative: ColourScheme,
    /// Colour for exact zero (light green: 0x80, 0xff, 0x80)
    pub zero: ColourScheme,
    /// Colour for undefined states (light magenta: 0xff, 0x80, 0xff)
    pub undefined: ColourScheme,
    /// Colour for positive (integer) exponents (light yellow: 0xff, 0xff, 0x80)
    pub integer_exponent: ColourScheme,
    /// Colour for negative (fractional) exponents (light cyan: 0x80, 0xff, 0xff)
    pub fractional_exponent: ColourScheme,
}

/// Global colour palette used for all debug formatting.
///
/// This constant defines the default colour scheme applied when formatting
/// Scalar and Circle values with `{:#?}`. Terminal support for 24-bit RGB
/// colours is required to see the colours properly.
pub const COLOURS: ScalarColours = ScalarColours {
    normal_positive: ColourScheme {
        colour: [0xff, 0x80, 0x80],
    },
    normal_negative: ColourScheme {
        colour: [0x80, 0x80, 0xff],
    },
    exploded_positive: ColourScheme {
        colour: [0xff, 0x00, 0x00],
    },
    exploded_negative: ColourScheme {
        colour: [0x00, 0x00, 0xff],
    },
    vanished_positive: ColourScheme {
        colour: [0xff, 0x00, 0x00],
    },
    vanished_negative: ColourScheme {
        colour: [0x00, 0x00, 0xff],
    },
    zero: ColourScheme {
        colour: [0x80, 0xff, 0x80],
    },
    undefined: ColourScheme {
        colour: [0xff, 0x80, 0xff],
    },
    integer_exponent: ColourScheme {
        colour: [0xff, 0xff, 0x80],
    },
    fractional_exponent: ColourScheme {
        colour: [0x80, 0xff, 0xff],
    },
};
