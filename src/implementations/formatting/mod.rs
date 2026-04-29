//! # Formatting Module
//!
//! This module provides comprehensive formatting implementations for `Scalar` and `Circle` types. All formatting operations use Spirix arithmetic directly to extract digits in any base from 2-36, without requiring conversion to u8 or bitmask operations.
//!
//! ## Key Features
//!
//! - **Any Base Support (2-36)**: The formatter can represent numbers in any base by using the precision formatter (e.g., `format!("{:.16}", value)` for base-16/hexadecimal)
//! - **Any Length**: The width formatter controls how many digits to display (e.g., `format!("{:10}", value)`)
//! - **Scientific Notation**: Automatically switches to scientific notation for very large or very small numbers
//! - **Direct Arithmetic**: Uses Spirix operations (`floor`, division, multiplication) to extract each digit rather than converting to primitive types and doing bitmask operations
//! - **Debug Formatting**: Binary representation with colourized output showing internal state
//!
//! ## Format Specifiers
//!
//! ### Display Formatting (`{}`)
//!
//! - **Default**: Base-10 with automatic precision based on fraction bits
//! - **Precision (base)**: `{:.B}` where B is the base (2-36)
//!   - Examples: `{:.2}` (binary), `{:.16}` (hex), `{:.36}` (base-36)
//! - **Width (digits)**: `{:W}` where W is the number of digits to display
//!   - Example: `{:20}` (display 20 digits)
//! - **Combined**: `{:W.B}` for both width and base
//!   - Example: `{:16.16}` (16 hex digits)
//!
//! ### Debug Formatting (`{:?}` and `{:#?}`)
//!
//! - **Plain (`{:?}`)**: Raw binary representation of fraction and exponent bits
//! - **Fancy (`{:#?}`)**: Colourized binary with special characters for different states
//!
//! ## Module Structure
//!
//! - `scalar`: Display and Debug formatting for Scalar types
//! - `circle`: Display and Debug formatting for Circle types
//! - `colours`: Colour schemes for debug output visualization

mod circle;
mod colours;
mod scalar;

pub use colours::{ColourScheme, ScalarColours, COLOURS};
