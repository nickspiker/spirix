// operators/circle_rust.rs
use crate::constants::{CircleConstants, ScalarConstants};
use crate::core::integer::*;
use crate::{operators::*, Circle, Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

// =====================================================
// Circle + Rust = Circle operations
// =====================================================
macro_rules! impl_circle_op_rust_to_circle {
    ($trait:ident, $method:ident, $circle_method:ident, $($t:ty),*) => {
        $(
            // &Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // &mut Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &mut Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    (&self).$circle_method(&scalar_rhs)
                }
            }
        )*
    }
}

// Implement standard arithmetic operations (returning Circle)
impl_circle_op_rust_to_circle!(
    Add,
    add,
    circle_add_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_op_rust_to_circle!(Add, add, circle_add_scalar, f32, f64);

impl_circle_op_rust_to_circle!(
    Sub,
    sub,
    circle_subtract_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_op_rust_to_circle!(Sub, sub, circle_subtract_scalar, f32, f64);

impl_circle_op_rust_to_circle!(
    Mul,
    mul,
    circle_multiply_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_op_rust_to_circle!(Mul, mul, circle_multiply_scalar, f32, f64);

impl_circle_op_rust_to_circle!(
    Div,
    div,
    circle_divide_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_op_rust_to_circle!(Div, div, circle_divide_scalar, f32, f64);

// =====================================================
// Circle + Rust = Scalar operations (for Rem)
// =====================================================
macro_rules! impl_circle_op_rust_to_scalar {
    ($trait:ident, $method:ident, $circle_method:ident, $($t:ty),*) => {
        $(
            // &Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Scalar<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // &mut Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &mut Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Scalar<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Scalar<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    (&self).$circle_method(&scalar_rhs)
                }
            }
        )*
    }
}

// Implement remainder operation (returns Scalar)
impl_circle_op_rust_to_scalar!(
    Rem,
    rem,
    circle_modulus_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_op_rust_to_scalar!(Rem, rem, circle_modulus_scalar, f32, f64);

// =====================================================
// Circle += Rust operations
// =====================================================
macro_rules! impl_circle_assign_op_rust {
    ($trait:ident, $method:ident, $circle_method:ident, $($t:ty),*) => {
        $(
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn $method(&mut self, rhs: $t) {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    *self = self.$circle_method(&scalar_rhs);
                }
            }
        )*
    }
}

// Implement assignment operations
impl_circle_assign_op_rust!(
    AddAssign,
    add_assign,
    circle_add_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_assign_op_rust!(AddAssign, add_assign, circle_add_scalar, f32, f64);

impl_circle_assign_op_rust!(
    SubAssign,
    sub_assign,
    circle_subtract_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_assign_op_rust!(SubAssign, sub_assign, circle_subtract_scalar, f32, f64);

impl_circle_assign_op_rust!(
    MulAssign,
    mul_assign,
    circle_multiply_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_assign_op_rust!(MulAssign, mul_assign, circle_multiply_scalar, f32, f64);

impl_circle_assign_op_rust!(
    DivAssign,
    div_assign,
    circle_divide_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_assign_op_rust!(DivAssign, div_assign, circle_divide_scalar, f32, f64);

// =====================================================
// Circle Shift Operations
// =====================================================
macro_rules! impl_circle_shift_op {
    ($trait:ident, $method:ident, $circle_method:ident, $($rhs_type:ty),*) => {
        $(
            // &Circle op shift
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$rhs_type> for &Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $rhs_type) -> Self::Output {
                    let shift: E = rhs.saturate();
                    self.$circle_method(&shift)
                }
            }

            // &mut Circle op shift
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$rhs_type> for &mut Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $rhs_type) -> Self::Output {
                    let shift: E = rhs.saturate();
                    self.$circle_method(&shift)
                }
            }

            // Circle op shift
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$rhs_type> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(self, rhs: $rhs_type) -> Self::Output {
                    let shift: E = rhs.saturate();
                    (&self).$circle_method(&shift)
                }
            }
        )*
    };
}

// Implement left and right shift operations
impl_circle_shift_op!(
    Shl,
    shl,
    circle_shl_integer,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize
);

impl_circle_shift_op!(
    Shr,
    shr,
    circle_shr_integer,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize
);

// =====================================================
// Circle Shift Assignment Operations
// =====================================================
macro_rules! impl_circle_shift_assign_op {
    ($trait:ident, $method:ident, $circle_method:ident, $($rhs_type:ty),*) => {
        $(
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$rhs_type> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn $method(&mut self, rhs: $rhs_type) {
                    let shift: E = rhs.saturate();
                    *self = self.$circle_method(&shift);
                }
            }
        )*
    };
}

// Implement shift assignment operations
impl_circle_shift_assign_op!(
    ShlAssign,
    shl_assign,
    circle_shl_integer,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize
);

impl_circle_shift_assign_op!(
    ShrAssign,
    shr_assign,
    circle_shr_integer,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize
);
// =====================================================
// Circle Powerer Operations
// =====================================================
macro_rules! impl_circle_exponent_rust {
    ($trait:ident, $method:ident, $circle_method:ident, $($t:ty),*) => {
        $(
            // &Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(&self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // &mut Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for &mut Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(&self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$circle_method(&scalar_rhs)
                }
            }

            // Circle op RustType
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > $trait<$t> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                type Output = Circle<F, E>;

                fn $method(&self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    (&self).$circle_method(&scalar_rhs)
                }
            }
        )*
    }
}

impl_circle_exponent_rust!(
    Modulo,
    modulo,
    circle_modulo_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_exponent_rust!(Modulo, modulo, circle_modulo_scalar, f32, f64);
impl_circle_exponent_rust!(
    Power,
    pow,
    circle_power_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_exponent_rust!(Power, pow, circle_power_scalar, f32, f64);
impl_circle_exponent_rust!(
    Logarithm,
    log,
    circle_logarithm_scalar,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);
impl_circle_exponent_rust!(Logarithm, log, circle_logarithm_scalar, f32, f64);

// =====================================================
// Circle comparison with Rust primitives
// =====================================================
macro_rules! impl_circle_cmp_rust {
    ($($t:ty),*) => {
        $(
            // Circle == primitive
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > PartialEq<$t> for Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn eq(&self, other: &$t) -> bool {
                    let circle_other = Circle::<F, E>::from(*other);
                    self.equals(&circle_other)
                }
            }

            // &Circle == primitive
            impl<
                'a,
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > PartialEq<$t> for &'a Circle<F, E>
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn eq(&self, other: &$t) -> bool {
                    let circle_other = Circle::<F, E>::from(*other);
                    self.equals(&circle_other)
                }
            }

            // primitive == Circle
            impl<
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > PartialEq<Circle<F, E>> for $t
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn eq(&self, other: &Circle<F, E>) -> bool {
                    let circle_self = Circle::<F, E>::from(*self);
                    circle_self.equals(other)
                }
            }

            // primitive == &Circle
            impl<
                'a,
                F: Integer
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
                E: Integer
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>
                    + WrappingNeg
                    + WrappingAdd
                    + WrappingMul
                    + WrappingSub,
            > PartialEq<&'a Circle<F, E>> for $t
            where
                Circle<F, E>: CircleConstants,
                Scalar<F, E>: ScalarConstants,
                u8: AsPrimitive<F>,
                u16: AsPrimitive<F>,
                u32: AsPrimitive<F>,
                u64: AsPrimitive<F>,
                u128: AsPrimitive<F>,
                usize: AsPrimitive<F>,
                i8: AsPrimitive<F>,
                i16: AsPrimitive<F>,
                i32: AsPrimitive<F>,
                i64: AsPrimitive<F>,
                i128: AsPrimitive<F>,
                isize: AsPrimitive<F>,
                I256: From<F>,
                u8: AsPrimitive<E>,
                u16: AsPrimitive<E>,
                u32: AsPrimitive<E>,
                u64: AsPrimitive<E>,
                u128: AsPrimitive<E>,
                usize: AsPrimitive<E>,
                i8: AsPrimitive<E>,
                i16: AsPrimitive<E>,
                i32: AsPrimitive<E>,
                i64: AsPrimitive<E>,
                i128: AsPrimitive<E>,
                isize: AsPrimitive<E>,
                I256: From<E>,
            {
                fn eq(&self, other: &&'a Circle<F, E>) -> bool {
                    let circle_self = Circle::<F, E>::from(*self);
                    circle_self.equals(other)
                }
            }
        )*
    };
}

impl_circle_cmp_rust!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_circle_cmp_rust!(f32, f64);
