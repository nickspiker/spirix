use crate::core::integer::*;
use crate::{operators::*, Circle, CircleConstants, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

macro_rules! impl_circle_op_scalar {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                (&self).$circle_method(rhs)
            }
        }

        // &Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &Circle op Scalar
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
            > $trait<Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$circle_method(&rhs)
            }
        }

        // &mut Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op Scalar
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
            > $trait<Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                self.$circle_method(&rhs)
            }
        }

        // Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Self::Output {
                (&self).$circle_method(rhs)
            }
        }

        // Circle op Scalar
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
            > $trait<Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Self::Output {
                (&self).$circle_method(&rhs)
            }
        }
    };
}

impl_circle_op_scalar!(Add, add, circle_add_scalar);
impl_circle_op_scalar!(Sub, sub, circle_subtract_scalar);
impl_circle_op_scalar!(Mul, mul, circle_multiply_scalar);
impl_circle_op_scalar!(Div, div, circle_divide_scalar);
macro_rules! impl_circle_op_scalar_to_scalar {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // &Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(rhs)
            }
        }

        // &Circle op Scalar
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
            > $trait<Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(&rhs)
            }
        }

        // &mut Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op Scalar
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
            > $trait<Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(&rhs)
            }
        }

        // Circle op &Scalar
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
            > $trait<&Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &Scalar<F, E>) -> Scalar<F, E> {
                (&self).$circle_method(rhs)
            }
        }

        // Circle op Scalar
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
            > $trait<Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: Scalar<F, E>) -> Scalar<F, E> {
                (&self).$circle_method(&rhs)
            }
        }
        // Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Scalar<F, E> {
                (&self).$circle_method(rhs)
            }
        }

        // &Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(self, rhs: &mut Scalar<F, E>) -> Scalar<F, E> {
                self.$circle_method(rhs)
            }
        }
    };
}

impl_circle_op_scalar_to_scalar!(Rem, rem, circle_modulus_scalar);

macro_rules! impl_circle_math_op_scalar {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // 1. Owned Circle, Owned Scalar
        #[allow(private_bounds)]
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
            > $trait<Scalar<F, E>> for Circle<F, E>
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$circle_method(&scalar)
            }
        }

        // 2. Owned Circle, Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&Scalar<F, E>> for Circle<F, E>
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }

        // 3. Owned Circle, Mutably Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&mut Scalar<F, E>> for Circle<F, E>
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }

        // 4. Referenced Circle, Owned Scalar
        #[allow(private_bounds)]
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
            > $trait<Scalar<F, E>> for &Circle<F, E>
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$circle_method(&scalar)
            }
        }

        // 5. Referenced Circle, Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&Scalar<F, E>> for &Circle<F, E>
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }

        // 6. Referenced Circle, Mutably Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&mut Scalar<F, E>> for &Circle<F, E>
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }

        // 7. Mutably Referenced Circle, Owned Scalar
        #[allow(private_bounds)]
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
            > $trait<Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(&self, scalar: Scalar<F, E>) -> Self::Output {
                self.$circle_method(&scalar)
            }
        }

        // 8. Mutably Referenced Circle, Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(&self, scalar: &Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }

        // 9. Mutably Referenced Circle, Mutably Referenced Scalar
        #[allow(private_bounds)]
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
            > $trait<&mut Scalar<F, E>> for &mut Circle<F, E>
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
            fn $method(&self, scalar: &mut Scalar<F, E>) -> Self::Output {
                self.$circle_method(scalar)
            }
        }
    };
}

impl_circle_math_op_scalar!(Modulo, modulo, circle_modulo_scalar);
impl_circle_math_op_scalar!(Power, pow, circle_power_scalar);
impl_circle_math_op_scalar!(Logarithm, log, circle_logarithm_scalar);
macro_rules! impl_circle_assign_op_scalar {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // Circle op= &Scalar
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
            > $trait<&Scalar<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &Scalar<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= &mut Scalar
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
            > $trait<&mut Scalar<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &mut Scalar<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= Scalar
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
            > $trait<Scalar<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: Scalar<F, E>) {
                *self = self.$circle_method(&rhs);
            }
        }
    };
}

impl_circle_assign_op_scalar!(AddAssign, add_assign, circle_add_scalar);
impl_circle_assign_op_scalar!(SubAssign, sub_assign, circle_subtract_scalar);
impl_circle_assign_op_scalar!(MulAssign, mul_assign, circle_multiply_scalar);
impl_circle_assign_op_scalar!(DivAssign, div_assign, circle_divide_scalar);
// impl_circle_assign_op_scalar!(RemAssign, rem_assign, circle_modulus_scalar); // Not implemented as % returns a Scalar
