use crate::core::integer::{FullInt, Integer};
use crate::{
    operators::*, Circle, CircleConstants, ExponentConstants, FractionConstants, Scalar,
    ScalarConstants,
};
use i256::I256;
use num_traits::AsPrimitive;
use std::ops::*;
macro_rules! impl_circle_op {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // &Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for &Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &mut Circle<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for &mut Circle<F, E>
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
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for &mut Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for Circle<F, E>
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
                (&self).$circle_method(rhs)
            }
        }

        // Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &mut Circle<F, E>) -> Self::Output {
                (&self).$circle_method(rhs)
            }
        }

        // Circle op Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                (&self).$circle_method(&rhs)
            }
        }

        // &Circle op Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<Circle<F, E>> for &Circle<F, E>
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
                self.$circle_method(&rhs)
            }
        }
    };
}

impl_circle_op!(Add, add, circle_add_circle);
impl_circle_op!(Sub, sub, circle_subtract_circle);
impl_circle_op!(Mul, mul, circle_multiply_circle);
impl_circle_op!(Div, div, circle_divide_circle);
impl_circle_op!(Rem, rem, circle_modulus_circle); // %

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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Modulo<Circle<F, E>> for Circle<F, E>
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

    fn modulo(&self, rhs: Circle<F, E>) -> Self::Output {
        self.circle_modulo_circle(&rhs)
    }
}
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Modulo<Circle<F, E>> for &Circle<F, E>
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

    fn modulo(&self, rhs: Circle<F, E>) -> Self::Output {
        self.circle_modulo_circle(&rhs)
    }
}
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Modulo<Circle<F, E>> for &mut Circle<F, E>
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

    fn modulo(&self, rhs: Circle<F, E>) -> Self::Output {
        self.circle_modulo_circle(&rhs)
    }
}

macro_rules! impl_circle_assign_op {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // Circle op= &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &Circle<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &mut Circle<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= Scalar
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: Circle<F, E>) {
                *self = self.$circle_method(&rhs);
            }
        }
    };
}

impl_circle_assign_op!(AddAssign, add_assign, circle_add_circle);
impl_circle_assign_op!(SubAssign, sub_assign, circle_subtract_circle);
impl_circle_assign_op!(MulAssign, mul_assign, circle_multiply_circle);
impl_circle_assign_op!(DivAssign, div_assign, circle_divide_circle);
impl_circle_assign_op!(RemAssign, rem_assign, circle_modulus_circle);
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<Circle<F, E>> for Circle<F, E>
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
    fn pow(&self, circle: Circle<F, E>) -> Self::Output {
        self.circle_power_circle(&circle)
    }
}

// Implement for &Circle -> Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<&Circle<F, E>> for Circle<F, E>
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
    fn pow(&self, circle: &Circle<F, E>) -> Self::Output {
        self.circle_power_circle(circle)
    }
}

// Implement for Circle -> &Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<Circle<F, E>> for &Circle<F, E>
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
    fn pow(&self, circle: Circle<F, E>) -> Self::Output {
        self.circle_power_circle(&circle)
    }
}

// Implement for &Circle -> &Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<&Circle<F, E>> for &Circle<F, E>
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
    fn pow(&self, circle: &Circle<F, E>) -> Self::Output {
        self.circle_power_circle(circle)
    }
}
// &mut Circle op Circle
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<Circle<F, E>> for &mut Circle<F, E>
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
    fn pow(&self, circle: Circle<F, E>) -> Self::Output {
        self.circle_power_circle(&circle)
    }
}

// &mut Circle op &mut Circle
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Power<&mut Circle<F, E>> for &mut Circle<F, E>
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
    fn pow(&self, circle: &mut Circle<F, E>) -> Self::Output {
        self.circle_power_circle(circle)
    }
}
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<Circle<F, E>> for Circle<F, E>
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
    fn log(&self, circle: Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(&circle)
    }
}

// Implement for &Circle -> Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<&Circle<F, E>> for Circle<F, E>
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
    fn log(&self, circle: &Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(circle)
    }
}

// Implement for Circle -> &Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<Circle<F, E>> for &Circle<F, E>
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
    fn log(&self, circle: Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(&circle)
    }
}

// Implement for &Circle -> &Circle
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<&Circle<F, E>> for &Circle<F, E>
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
    fn log(&self, circle: &Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(circle)
    }
}
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<Circle<F, E>> for &mut Circle<F, E>
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

    fn log(&self, rhs: Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(&rhs)
    }
}

// &mut Circle op &mut Circle
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Logarithm<&mut Circle<F, E>> for &mut Circle<F, E>
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
    fn log(&self, circle: &mut Circle<F, E>) -> Self::Output {
        self.circle_logarithm_circle(circle)
    }
}

macro_rules! impl_circle_bitwise_op {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // &Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for &Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for &Circle<F, E>
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
            fn $method(self, rhs: &mut Circle<F, E>) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for &mut Circle<F, E>
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
                self.$circle_method(rhs)
            }
        }

        // &mut Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for &mut Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                self.$circle_method(rhs)
            }
        }

        // Circle op &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for Circle<F, E>
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
                (&self).$circle_method(rhs)
            }
        }

        // Circle op &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for Circle<F, E>
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
            fn $method(self, rhs: &mut Circle<F, E>) -> Self::Output {
                (&self).$circle_method(rhs)
            }
        }

        // Circle op Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait for Circle<F, E>
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
            fn $method(self, rhs: Self) -> Self::Output {
                (&self).$circle_method(&rhs)
            }
        }
        // &Circle op Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<Circle<F, E>> for &Circle<F, E>
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
                self.$circle_method(&rhs)
            }
        }
    };
}

impl_circle_bitwise_op!(BitAnd, bitand, aligned_and);
impl_circle_bitwise_op!(BitOr, bitor, aligned_or);
impl_circle_bitwise_op!(BitXor, bitxor, aligned_xor);

macro_rules! impl_circle_bitwise_assign_op {
    ($trait:ident, $method:ident, $circle_method:ident) => {
        // Circle op= &Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &Circle<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= &mut Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<&mut Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: &mut Circle<F, E>) {
                *self = self.$circle_method(rhs);
            }
        }

        // Circle op= Circle
        impl<
                F: Integer
                    + FractionConstants
                    + FullInt
                    + Shl<isize, Output = F>
                    + Shr<isize, Output = F>
                    + Shl<F, Output = F>
                    + Shr<F, Output = F>
                    + Shl<E, Output = F>
                    + Shr<E, Output = F>,
                E: Integer
                    + ExponentConstants
                    + FullInt
                    + Shl<isize, Output = E>
                    + Shr<isize, Output = E>
                    + Shl<E, Output = E>
                    + Shr<E, Output = E>
                    + Shl<F, Output = E>
                    + Shr<F, Output = E>,
            > $trait<Circle<F, E>> for Circle<F, E>
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
            fn $method(&mut self, rhs: Circle<F, E>) {
                *self = self.$circle_method(&rhs);
            }
        }
    };
}

impl_circle_bitwise_assign_op!(BitAndAssign, bitand_assign, aligned_and);
impl_circle_bitwise_assign_op!(BitOrAssign, bitor_assign, aligned_or);
impl_circle_bitwise_assign_op!(BitXorAssign, bitxor_assign, aligned_xor);

impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > PartialEq for Circle<F, E>
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
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
