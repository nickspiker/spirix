pub mod circle_circle;
#[cfg(feature = "ieee")]
pub mod circle_rust;
pub mod circle_scalar;
pub mod rust_circle;
pub mod rust_scalar;
pub mod scalar_circle;
pub mod scalar_rust;
pub mod scalar_scalar;
include!("../generated_conversions.rs");
