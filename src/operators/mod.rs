// src/operators/mod.rs
pub mod circle;
pub mod circle_circle;
pub mod circle_rust;
pub mod circle_scalar;
pub mod rust_circle;
pub mod rust_scalar;
pub mod scalar;
pub mod scalar_circle;
pub mod scalar_rust;
pub mod scalar_scalar;

pub trait Modulo<Rhs = Self> {
    type Output;
    fn modulo(&self, rhs: Rhs) -> Self::Output;
}

pub trait Power<Rhs = Self> {
    type Output;
    fn pow(&self, rhs: Rhs) -> Self::Output;
}

pub trait Logarithm<Rhs = Self> {
    type Output;
    fn log(&self, rhs: Rhs) -> Self::Output;
}