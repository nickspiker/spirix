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

pub trait Min<Rhs = Self> {
    type Output;
    fn min(&self, rhs: Rhs) -> Self::Output;
}

pub trait Max<Rhs = Self> {
    type Output;
    fn max(&self, rhs: Rhs) -> Self::Output;
}

pub trait Clamp<Min, Max> {
    type Output;
    fn clamp(&self, min: Min, max: Max) -> Self::Output;
}
