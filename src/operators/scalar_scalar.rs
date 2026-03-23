use crate::core::integer::{FullInt, Integer};
use crate::{operators::*, ExponentConstants, FractionConstants, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use core::cmp::Ordering;
use core::ops::*;
macro_rules! impl_scalar_op {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // &Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }

        // &mut Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }
        // &Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for &Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &mut Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &mut Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                (&self).$scalar_method(rhs)
            }
        }

        // Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                (&self).$scalar_method(rhs)
            }
        }

        // Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                (&self).$scalar_method(&rhs)
            }
        }
    };
}

impl_scalar_op!(Add, add, scalar_add_scalar);
impl_scalar_op!(Sub, sub, scalar_subtract_scalar);
impl_scalar_op!(Mul, mul, scalar_multiply_scalar);
impl_scalar_op!(Div, div, scalar_divide_scalar);
impl_scalar_op!(Rem, rem, scalar_modulus_scalar);

macro_rules! impl_scalar_assign_op {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // Scalar op= &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: &Scalar<F, E>) {
                *self = self.$scalar_method(rhs);
            }
        }

        // Scalar op= &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: &mut Scalar<F, E>) {
                *self = self.$scalar_method(rhs);
            }
        }

        // Scalar op= Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: Scalar<F, E>) {
                *self = self.$scalar_method(&rhs);
            }
        }
    };
}

impl_scalar_assign_op!(AddAssign, add_assign, scalar_add_scalar);
impl_scalar_assign_op!(SubAssign, sub_assign, scalar_subtract_scalar);
impl_scalar_assign_op!(MulAssign, mul_assign, scalar_multiply_scalar);
impl_scalar_assign_op!(DivAssign, div_assign, scalar_divide_scalar);
impl_scalar_assign_op!(RemAssign, rem_assign, scalar_modulus_scalar);
macro_rules! impl_scalar_math_op {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // Scalar op Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&scalar)
            }
        }

        // Scalar op &Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }

        // Scalar op &mut Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }

        // &Scalar op Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&scalar)
            }
        }

        // &Scalar op &Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }

        // &Scalar op &mut Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }

        // &mut Scalar op Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&scalar)
            }
        }

        // &mut Scalar op &Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }

        // &mut Scalar op &mut Scalar
        #[allow(private_bounds)]
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$scalar_method(scalar)
            }
        }
    };
}

// Use the macro to implement Power and Logarithm traits
impl_scalar_math_op!(Power, pow, scalar_power_scalar);
impl_scalar_math_op!(Logarithm, log, scalar_logarithm_scalar);
macro_rules! impl_scalar_bitwise_op {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // &Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for &Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &mut Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &mut Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // Scalar op &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                (&self).$scalar_method(rhs)
            }
        }

        // Scalar op &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                (&self).$scalar_method(rhs)
            }
        }
        // &Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &Scalar<F, E>
        where
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }

        // &mut Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for &mut Scalar<F, E>
        where
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }
        // Scalar op Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait for Scalar<F, E>
        where
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
            fn $method(self, rhs: Self) -> Self::Output {
                (&self).$scalar_method(&rhs)
            }
        }
    };
}
impl_scalar_bitwise_op!(BitAnd, bitand, aligned_and);
impl_scalar_bitwise_op!(BitOr, bitor, aligned_or);
impl_scalar_bitwise_op!(BitXor, bitxor, aligned_xor);

macro_rules! impl_scalar_bitwise_assign_op {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // Scalar op= &Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: &Scalar<F, E>) {
                *self = self.$scalar_method(rhs);
            }
        }

        // Scalar op= &mut Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<&mut Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: &mut Scalar<F, E>) {
                *self = self.$scalar_method(rhs);
            }
        }

        // Scalar op= Scalar
        impl<
                F: Integer
                    + FractionConstants
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
                    + ExponentConstants
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
            > $trait<Scalar<F, E>> for Scalar<F, E>
        where
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
            fn $method(&mut self, rhs: Scalar<F, E>) {
                *self = self.$scalar_method(&rhs);
            }
        }
    };
}

// Implement bitwise assignment operators
impl_scalar_bitwise_assign_op!(BitAndAssign, bitand_assign, aligned_and);
impl_scalar_bitwise_assign_op!(BitOrAssign, bitor_assign, aligned_or);
impl_scalar_bitwise_assign_op!(BitXorAssign, bitxor_assign, aligned_xor);
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialEq for Scalar<F, E>
where
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
    fn eq(&self, other: &Self) -> bool {
        matches!(self.compare(other), Some(Ordering::Equal))
    }
}

// PartialEq implementation for different reference combinations
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialEq<&Scalar<F, E>> for Scalar<F, E>
where
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
    fn eq(&self, other: &&Self) -> bool {
        matches!(self.compare(*other), Some(Ordering::Equal))
    }
}

impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialEq<Scalar<F, E>> for &Scalar<F, E>
where
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
    fn eq(&self, other: &Scalar<F, E>) -> bool {
        matches!(self.compare(other), Some(Ordering::Equal))
    }
}

// PartialOrd implementation
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialOrd for Scalar<F, E>
where
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.compare(other)
    }
}
// Scalar == &mut Scalar
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialEq<&mut Scalar<F, E>> for Scalar<F, E>
where
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
    fn eq(&self, other: &&mut Self) -> bool {
        matches!(self.compare(*other), Some(core::cmp::Ordering::Equal))
    }
}

// &mut Scalar == Scalar
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialEq<Scalar<F, E>> for &mut Scalar<F, E>
where
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
    fn eq(&self, other: &Scalar<F, E>) -> bool {
        matches!((*self).compare(other), Some(core::cmp::Ordering::Equal))
    }
}
// Value vs reference
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialOrd<&Scalar<F, E>> for Scalar<F, E>
where
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
    fn partial_cmp(&self, other: &&Scalar<F, E>) -> Option<core::cmp::Ordering> {
        self.compare(*other)
    }
}

// Reference vs value
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialOrd<Scalar<F, E>> for &Scalar<F, E>
where
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
    fn partial_cmp(&self, other: &Scalar<F, E>) -> Option<core::cmp::Ordering> {
        (*self).compare(other)
    }
}

// With mutable references (these are likely to be needed)
impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialOrd<&mut Scalar<F, E>> for Scalar<F, E>
where
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
    fn partial_cmp(&self, other: &&mut Scalar<F, E>) -> Option<core::cmp::Ordering> {
        self.compare(*other)
    }
}

impl<
        F: Integer
            + FractionConstants
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
            + ExponentConstants
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
    > PartialOrd<Scalar<F, E>> for &mut Scalar<F, E>
where
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
    fn partial_cmp(&self, other: &Scalar<F, E>) -> Option<core::cmp::Ordering> {
        (*self).compare(other)
    }
}
