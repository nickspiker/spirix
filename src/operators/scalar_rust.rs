// src/operators/scalar_rust.rs
use crate::core::integer::{FullInt, IntConvert};
use crate::{operators::*, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use core::ops::*;

// Scalar + RustType operations
macro_rules! impl_scalar_op_rust {
    ($trait:ident, $method:ident, $scalar_method:ident, $($t:ty),*) => {
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
     > $trait<$t> for &Scalar<F, E>
            where
            F: FractionConstants,
            E: ExponentConstants,
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
            I256: From<F>,
            I256: From<E>,
        {
                type Output = Scalar<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.$scalar_method(&scalar_rhs)
                }
            }

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
        > $trait<$t> for Scalar<F, E>
               where
               F: FractionConstants,
               E: ExponentConstants,
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
               I256: From<F>,
               I256: From<E>,
           {
                type Output = Scalar<F, E>;

                fn $method(self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    (&self).$scalar_method(&scalar_rhs)
                }
            }
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
        > $trait<$t> for &mut Scalar<F, E>
        where
            F: FractionConstants,
            E: ExponentConstants,
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
            I256: From<F>,
            I256: From<E>,
        {
            type Output = Scalar<F, E>;

            fn $method(self, rhs: $t) -> Self::Output {
                let scalar_rhs = Scalar::<F, E>::from(rhs);
                self.$scalar_method(&scalar_rhs)
            }
        }
        )*
    }
}

impl_scalar_op_rust!(
    Add,
    add,
    scalar_add_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_op_rust!(Add, add, scalar_add_scalar, f32, f64);
impl_scalar_op_rust!(
    Sub,
    sub,
    scalar_subtract_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_op_rust!(Sub, sub, scalar_subtract_scalar, f32, f64);
impl_scalar_op_rust!(
    Mul,
    mul,
    scalar_multiply_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_op_rust!(Mul, mul, scalar_multiply_scalar, f32, f64);
impl_scalar_op_rust!(
    Div,
    div,
    scalar_divide_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_op_rust!(Div, div, scalar_divide_scalar, f32, f64);
impl_scalar_op_rust!(
    Rem,
    rem,
    scalar_modulus_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_op_rust!(Rem, rem, scalar_modulus_scalar, f32, f64);

// Add AssignOp implementations for primitive types
macro_rules! impl_scalar_assign_op_rust {
    ($trait:ident, $method:ident, $scalar_method:ident, $($t:ty),*) => {
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
     > $trait<$t> for Scalar<F, E>
     where
     F: FractionConstants,
     E: ExponentConstants,
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
     I256: From<F>,
     I256: From<E>,
 {
                fn $method(&mut self, rhs: $t) {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    *self = self.$scalar_method(&scalar_rhs);
                }
            }
        )*
    }
}

// Implement assignment operations
impl_scalar_assign_op_rust!(
    AddAssign, add_assign, scalar_add_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_assign_op_rust!(AddAssign, add_assign, scalar_add_scalar, f32, f64);
impl_scalar_assign_op_rust!(
    SubAssign, sub_assign, scalar_subtract_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_assign_op_rust!(SubAssign, sub_assign, scalar_subtract_scalar, f32, f64);
impl_scalar_assign_op_rust!(
    MulAssign, mul_assign, scalar_multiply_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_assign_op_rust!(MulAssign, mul_assign, scalar_multiply_scalar, f32, f64);
impl_scalar_assign_op_rust!(
    DivAssign, div_assign, scalar_divide_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_assign_op_rust!(DivAssign, div_assign, scalar_divide_scalar, f32, f64);
impl_scalar_assign_op_rust!(
    RemAssign, rem_assign, scalar_modulus_scalar,
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_assign_op_rust!(RemAssign, rem_assign, scalar_modulus_scalar, f32, f64);
// Power and Exp implementations for Scalar with primitive types
macro_rules! impl_scalar_power_rust {
    ($($t:ty),*) => {
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
     > Power<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn pow(&self, rhs: $t) -> Self::Output {
             let scalar_rhs = Scalar::<F, E>::from(rhs);
             self.scalar_power_scalar(&scalar_rhs)
         }
     }

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
 > Power<$t> for &Scalar<F, E>
 where
     F: FractionConstants,
     E: ExponentConstants,
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
     I256: From<F>,
     I256: From<E>,
 {
     type Output = Scalar<F, E>;

     fn pow(&self, rhs: $t) -> Self::Output {
         let scalar_rhs = Scalar::<F, E>::from(rhs);
         self.scalar_power_scalar(&scalar_rhs)
     }
 }
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
            > Power<$t> for &mut Scalar<F, E>
            where
                F: FractionConstants,
                E: ExponentConstants,
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
                I256: From<F>,
                I256: From<E>,
            {
                type Output = Scalar<F, E>;

                fn pow(&self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.scalar_power_scalar(&scalar_rhs)
                }
            }

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
            > Logarithm<$t> for &mut Scalar<F, E>
            where
                F: FractionConstants,
                E: ExponentConstants,
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
                I256: From<F>,
                I256: From<E>,
            {
                type Output = Scalar<F, E>;

                fn log(&self, rhs: $t) -> Self::Output {
                    let scalar_rhs = Scalar::<F, E>::from(rhs);
                    self.scalar_logarithm_scalar(&scalar_rhs)
                }
            }
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
     > Logarithm<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn log(&self, rhs: $t) -> Self::Output {
             let scalar_rhs = Scalar::<F, E>::from(rhs);
             self.scalar_logarithm_scalar(&scalar_rhs)
         }
     }

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
 > Logarithm<$t> for &Scalar<F, E>
 where
     F: FractionConstants,
     E: ExponentConstants,
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
     I256: From<F>,
     I256: From<E>,
 {
     type Output = Scalar<F, E>;

     fn log(&self, rhs: $t) -> Self::Output {
         let scalar_rhs = Scalar::<F, E>::from(rhs);
         self.scalar_logarithm_scalar(&scalar_rhs)
     }
 }
        )*
    }
}

// Implement power and exp for all primitive types
impl_scalar_power_rust!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_scalar_power_rust!(f32, f64);

// Min, Max and Clamp implementations for Scalar with primitive types
macro_rules! impl_scalar_comparison_rust {
    ($($t:ty),*) => {
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
     > Min<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn min(&self, rhs: $t) -> Self::Output {
             let scalar_rhs = Scalar::<F, E>::from(rhs);
             self.min(scalar_rhs)
         }
     }

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
     > Max<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn max(&self, rhs: $t) -> Self::Output {
             let scalar_rhs = Scalar::<F, E>::from(rhs);
             self.max(scalar_rhs)
         }
     }
        )*
    };
}

// Clamp implementation for Scalar with primitive types
macro_rules! impl_scalar_clamp_rust {
    ($($t:ty),*) => {
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
     > Clamp<$t, $t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn clamp(&self, min: $t, max: $t) -> Self::Output {
             let scalar_min = Scalar::<F, E>::from(min);
             let scalar_max = Scalar::<F, E>::from(max);
             self.clamp(scalar_min, scalar_max)
         }
     }
        )*
    };
}

// Implement min/max/clamp for all primitive types
impl_scalar_comparison_rust!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_scalar_comparison_rust!(f32, f64);
impl_scalar_clamp_rust!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
impl_scalar_clamp_rust!(f32, f64);
// Integer bit-shift implementations (self*2^integer) for Scalar with primitive types
macro_rules! impl_scalar_shift_rust {
    ($($t:ty),*) => {
        $(
              // &mut Scalar << primitive
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
          > Shl<$t> for &mut Scalar<F, E>
              where
              F: FractionConstants,
              E: ExponentConstants,
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
              I256: From<F>,
              I256: From<E>,
          {
              type Output = Scalar<F, E>;

              fn shl(self, rhs: $t) -> Self::Output {
                self.scalar_shl_integer(&rhs.saturate())
              }
          }
         // Owned << primitive
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
     > Shl<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn shl(self, rhs: $t) -> Self::Output {
                self.scalar_shl_integer(&rhs.saturate())
         }
     }

     // Reference << primitive
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
     > Shl<$t> for &Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn shl(self, rhs: $t) -> Self::Output {
            self.scalar_shl_integer(&rhs.saturate())
         }
     }
// &mut Scalar >> primitive
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
> Shr<$t> for &mut Scalar<F, E>
where
F: FractionConstants,
E: ExponentConstants,
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
I256: From<F>,
I256: From<E>,
{
type Output = Scalar<F, E>;

fn shr(self, rhs: $t) -> Self::Output {
        self.scalar_shr_integer(&rhs.saturate())
}
}
     // Owned >> primitive
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
     > Shr<$t> for Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn shr(self, rhs: $t) -> Self::Output {
        self.scalar_shr_integer(&rhs.saturate())
         }
     }

     // Reference >> primitive
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
     > Shr<$t> for &Scalar<F, E>
     where
         F: FractionConstants,
         E: ExponentConstants,
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
         I256: From<F>,
         I256: From<E>,
     {
         type Output = Scalar<F, E>;

         fn shr(self, rhs: $t) -> Self::Output {
        self.scalar_shr_integer(&rhs.saturate())
         }
     }
        )*
    }
}

impl_scalar_shift_rust!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Shift assignment operators
macro_rules! impl_scalar_shift_assign_rust {
    ($($t:ty),*) => {
        $(
            // Scalar <<= primitive
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
            > ShlAssign<$t> for Scalar<F, E>
                where
                F: FractionConstants,
                E: ExponentConstants,
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
                I256: From<F>,
                I256: From<E>,
            {
                fn shl_assign(&mut self, rhs: $t) {
                        *self=self.scalar_shl_integer(&rhs.saturate());
                }
            }

            // Scalar >>= primitive
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
            > ShrAssign<$t> for Scalar<F, E>
                where
                F: FractionConstants,
                E: ExponentConstants,
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
                I256: From<F>,
                I256: From<E>,
            {
                fn shr_assign(&mut self, rhs: $t) {
                        *self=self.scalar_shr_integer(&rhs.saturate());
                }
            }
        )*
    }
}

impl_scalar_shift_assign_rust!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Bitwise operations with primitives (Scalar OP primitive)
impl_scalar_op_rust!(
    BitAnd,
    bitand,
    aligned_and,
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

impl_scalar_op_rust!(
    BitOr, bitor, aligned_or, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl_scalar_op_rust!(
    BitXor,
    bitxor,
    aligned_xor,
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
