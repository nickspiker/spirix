use crate::core::integer::{FullInt, Integer};
use crate::{
    operators::*, Circle, CircleConstants, ExponentConstants, FractionConstants, Scalar,
    ScalarConstants,
};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

macro_rules! impl_scalar_op_circle {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // &Scalar op &Circle
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
            > $trait<&Circle<F, E>> for &Scalar<F, E>
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
            fn $method(self, rhs: &Circle<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &Scalar op Circle
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
            > $trait<Circle<F, E>> for &Scalar<F, E>
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
            fn $method(self, rhs: Circle<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }

        // &mut Scalar op &Circle
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
            > $trait<&Circle<F, E>> for &mut Scalar<F, E>
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
            fn $method(self, rhs: &Circle<F, E>) -> Self::Output {
                self.$scalar_method(rhs)
            }
        }

        // &mut Scalar op Circle
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
            > $trait<Circle<F, E>> for &mut Scalar<F, E>
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
            fn $method(self, rhs: Circle<F, E>) -> Self::Output {
                self.$scalar_method(&rhs)
            }
        }

        // Scalar op &Circle
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
            > $trait<&Circle<F, E>> for Scalar<F, E>
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
            fn $method(self, rhs: &Circle<F, E>) -> Self::Output {
                (&self).$scalar_method(rhs)
            }
        }

        // Scalar op Circle
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
            > $trait<Circle<F, E>> for Scalar<F, E>
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
            fn $method(self, rhs: Circle<F, E>) -> Self::Output {
                (&self).$scalar_method(&rhs)
            }
        }
    };
}

impl_scalar_op_circle!(Add, add, scalar_add_circle);
impl_scalar_op_circle!(Sub, sub, scalar_subtract_circle);
impl_scalar_op_circle!(Mul, mul, scalar_multiply_circle);
impl_scalar_op_circle!(Div, div, scalar_divide_circle);
impl_scalar_op_circle!(Rem, rem, scalar_modulus_circle);
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
    > Modulo<Circle<F, E>> for Scalar<F, E>
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
    fn modulo(&self, circle: Circle<F, E>) -> Circle<F, E> {
        self.scalar_modulo_circle(&circle)
    }
}
macro_rules! impl_scalar_math_op_circle {
    ($trait:ident, $method:ident, $scalar_method:ident) => {
        // 1. Owned Scalar, Owned Circle
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
            > $trait<Circle<F, E>> for Scalar<F, E>
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
            fn $method(&self, circle: Circle<F, E>) -> Circle<F, E> {
                self.$scalar_method(&circle)
            }
        }

        // 2. Owned Scalar, Referenced Circle
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
            > $trait<&Circle<F, E>> for Scalar<F, E>
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
            fn $method(&self, circle: &Circle<F, E>) -> Circle<F, E> {
                self.$scalar_method(circle)
            }
        }

        // 3. Referenced Scalar, Owned Circle
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
            > $trait<Circle<F, E>> for &Scalar<F, E>
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
            fn $method(&self, circle: Circle<F, E>) -> Circle<F, E> {
                self.$scalar_method(&circle)
            }
        }

        // 4. Referenced Scalar, Referenced Circle
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
            > $trait<&Circle<F, E>> for &Scalar<F, E>
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
            fn $method(&self, circle: &Circle<F, E>) -> Circle<F, E> {
                self.$scalar_method(circle)
            }
        }
    };
}

// Use the macro to implement Power and Logarithm traits
impl_scalar_math_op_circle!(Power, pow, scalar_power_circle);
impl_scalar_math_op_circle!(Logarithm, log, scalar_logarithm_circle);
