use crate::core::integer::FullInt;
use crate::core::integer::IntConvert;
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::AsPrimitive;
use num_traits::WrappingNeg;
use num_traits::WrappingAdd;
use num_traits::WrappingMul;
use num_traits::WrappingSub;
use std::ops::*;
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
    > Circle<F, E>
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
    pub(crate) fn aligned_and(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite()
                || other.is_infinite()
                || (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
            {
                return Self {
                    real: AND.prefix.sa(),
                    imaginary: AND.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.vanished() {
                let mut result = Self {
                    real: if self.real.is_negative() {
                        other.real
                    } else {
                        F::ZERO
                    },
                    imaginary: if self.imaginary.is_negative() {
                        other.imaginary
                    } else {
                        F::ZERO
                    },
                    exponent: other.exponent,
                };
                if result.real == F::ZERO && result.imaginary == F::ZERO {
                    result.exponent = E::AMBIGUOUS_EXPONENT;
                    return result;
                }
                if other.exploded() {
                    result.normalize_exploded();
                } else {
                    result.normalize();
                }
                return result;
            }
            if other.vanished() {
                let mut result = Self {
                    real: if other.real.is_negative() {
                        self.real
                    } else {
                        F::ZERO
                    },
                    imaginary: if other.imaginary.is_negative() {
                        self.imaginary
                    } else {
                        F::ZERO
                    },
                    exponent: self.exponent,
                };
                if result.real == F::ZERO && result.imaginary == F::ZERO {
                    result.exponent = E::AMBIGUOUS_EXPONENT;
                    return result;
                }
                if self.exploded() {
                    result.normalize_exploded();
                } else {
                    result.normalize();
                }
                return result;
            }
            if self.exploded() {
                let mut result = Self {
                    real: if other.real.is_negative() {
                        self.real
                    } else {
                        F::ZERO
                    },
                    imaginary: if other.imaginary.is_negative() {
                        self.imaginary
                    } else {
                        F::ZERO
                    },
                    exponent: self.exponent,
                };
                result.normalize_exploded();
                return result;
            }
            let mut result = Self {
                real: if self.real.is_negative() {
                    other.real
                } else {
                    F::ZERO
                },
                imaginary: if self.imaginary.is_negative() {
                    other.imaginary
                } else {
                    F::ZERO
                },
                exponent: other.exponent,
            };
            result.normalize_exploded();
            return result;
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            let mut result = Self {
                real: if small.real.is_negative() {
                    big.real
                } else {
                    F::ZERO
                },
                imaginary: if small.imaginary.is_negative() {
                    big.imaginary
                } else {
                    F::ZERO
                },
                exponent: big.exponent,
            };
            if result.real == F::ZERO && result.imaginary == F::ZERO {
                result.exponent = E::AMBIGUOUS_EXPONENT;
                return result;
            }
            result.normalize();
            return result;
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                let mut result = Self {
                    real: if small.real.is_negative() {
                        big.real
                    } else {
                        F::ZERO
                    },
                    imaginary: if small.imaginary.is_negative() {
                        big.imaginary
                    } else {
                        F::ZERO
                    },
                    exponent: big.exponent,
                };
                if result.real == F::ZERO && result.imaginary == F::ZERO {
                    result.exponent = E::AMBIGUOUS_EXPONENT;
                    return result;
                }
                result.normalize();
                return result;
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                let mut result = Self {
                    real: if small.real.is_negative() {
                        big.real
                    } else {
                        F::ZERO
                    },
                    imaginary: if small.imaginary.is_negative() {
                        big.imaginary
                    } else {
                        F::ZERO
                    },
                    exponent: big.exponent,
                };
                if result.real == F::ZERO && result.imaginary == F::ZERO {
                    result.exponent = E::AMBIGUOUS_EXPONENT;
                    return result;
                }
                result.normalize();
                return result;
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i16 = big.real.as_();
                big_r <<= shift;
                let small_r: i16 = small.real.as_();
                let real_result = big_r & small_r;
                let mut big_i: i16 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i16 = small.imaginary.as_();
                let imag_result = big_i & small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            16 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i32 = big.real.as_();
                big_r <<= shift;
                let small_r: i32 = small.real.as_();
                let real_result = big_r & small_r;
                let mut big_i: i32 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i32 = small.imaginary.as_();
                let imag_result = big_i & small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            32 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i64 = big.real.as_();
                big_r <<= shift;
                let small_r: i64 = small.real.as_();
                let real_result = big_r & small_r;
                let mut big_i: i64 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i64 = small.imaginary.as_();
                let imag_result = big_i & small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            64 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i128 = big.real.as_();
                big_r <<= shift;
                let small_r: i128 = small.real.as_();
                let real_result = big_r & small_r;
                let mut big_i: i128 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i128 = small.imaginary.as_();
                let imag_result = big_i & small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            128 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: I256 = big.real.into();
                big_r <<= shift;
                let small_r: I256 = small.real.into();
                let real_result = big_r & small_r;
                let mut big_i: I256 = big.imaginary.into();
                big_i <<= shift;
                let small_i: I256 = small.imaginary.into();
                let imag_result = big_i & small_i;
                if real_result == 0.into() && imag_result == 0.into() {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            _ => Self {
                real: GENERAL.prefix.sa(),
                imaginary: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    pub(crate) fn aligned_or(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite() || other.is_infinite() || (self.exploded() && other.exploded()) {
                return Self {
                    real: OR.prefix.sa(),
                    imaginary: OR.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.exploded() {
                if other.real.is_negative() && other.imaginary.is_negative() {
                    return *other;
                }
                if other.real.is_negative() {
                    let mut result = Scalar {
                        fraction: self.imaginary,
                        exponent: self.exponent,
                    };
                    result.normalize_exploded();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if other.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: self.real,
                        exponent: self.exponent,
                    };
                    result.normalize_exploded();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *self;
            }
            if other.exploded() {
                if self.real.is_negative() && self.imaginary.is_negative() {
                    return *self;
                }
                if self.real.is_negative() {
                    let mut result = Scalar {
                        fraction: other.imaginary,
                        exponent: other.exponent,
                    };
                    result.normalize_exploded();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if self.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: other.real,
                        exponent: other.exponent,
                    };
                    result.normalize_exploded();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *self;
            }
            if self.is_normal() {
                if other.real.is_negative() && other.imaginary.is_negative() {
                    return *other;
                }
                if other.real.is_negative() {
                    let mut result = Scalar {
                        fraction: self.imaginary,
                        exponent: self.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if other.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: self.real,
                        exponent: self.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *self;
            }
            if other.is_normal() {
                if self.real.is_negative() && self.imaginary.is_negative() {
                    return *self;
                }
                if self.real.is_negative() {
                    let mut result = Scalar {
                        fraction: other.imaginary,
                        exponent: other.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if self.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: other.real,
                        exponent: other.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *other;
            }
            return Self {
                real: OR.prefix.sa(),
                imaginary: OR.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            if small.real.is_negative() && small.imaginary.is_negative() {
                return *small;
            }
            if small.real.is_negative() {
                let mut result = Scalar {
                    fraction: big.imaginary,
                    exponent: big.exponent,
                };
                result.normalize();
                let mut negative_one: F = F::ZERO;
                negative_one = !negative_one;
                return Circle {
                    real: negative_one,
                    imaginary: result.fraction,
                    exponent: result.exponent,
                };
            }
            if small.imaginary.is_negative() {
                let mut result = Scalar {
                    fraction: big.real,
                    exponent: big.exponent,
                };
                result.normalize();
                let mut negative_one: F = F::ZERO;
                negative_one = !negative_one;
                return Circle {
                    real: result.fraction,
                    imaginary: negative_one,
                    exponent: result.exponent,
                };
            }
            return *big;
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                if small.real.is_negative() && small.imaginary.is_negative() {
                    return *small;
                }
                if small.real.is_negative() {
                    let mut result = Scalar {
                        fraction: big.imaginary,
                        exponent: big.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if small.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: big.real,
                        exponent: big.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *big;
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                if small.real.is_negative() && small.imaginary.is_negative() {
                    return *small;
                }
                if small.real.is_negative() {
                    let mut result = Scalar {
                        fraction: big.imaginary,
                        exponent: big.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: negative_one,
                        imaginary: result.fraction,
                        exponent: result.exponent,
                    };
                }
                if small.imaginary.is_negative() {
                    let mut result = Scalar {
                        fraction: big.real,
                        exponent: big.exponent,
                    };
                    result.normalize();
                    let mut negative_one: F = F::ZERO;
                    negative_one = !negative_one;
                    return Circle {
                        real: result.fraction,
                        imaginary: negative_one,
                        exponent: result.exponent,
                    };
                }
                return *big;
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i16 = big.real.as_();
                big_r <<= shift;
                let small_r: i16 = small.real.as_();
                let real_result = big_r | small_r;
                let mut big_i: i16 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i16 = small.imaginary.as_();
                let imag_result = big_i | small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            16 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i32 = big.real.as_();
                big_r <<= shift;
                let small_r: i32 = small.real.as_();
                let real_result = big_r | small_r;
                let mut big_i: i32 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i32 = small.imaginary.as_();
                let imag_result = big_i | small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            32 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i64 = big.real.as_();
                big_r <<= shift;
                let small_r: i64 = small.real.as_();
                let real_result = big_r | small_r;
                let mut big_i: i64 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i64 = small.imaginary.as_();
                let imag_result = big_i | small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            64 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i128 = big.real.as_();
                big_r <<= shift;
                let small_r: i128 = small.real.as_();
                let real_result = big_r | small_r;
                let mut big_i: i128 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i128 = small.imaginary.as_();
                let imag_result = big_i | small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            128 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: I256 = big.real.into();
                big_r <<= shift;
                let small_r: I256 = small.real.into();
                let real_result = big_r | small_r;
                let mut big_i: I256 = big.imaginary.into();
                big_i <<= shift;
                let small_i: I256 = small.imaginary.into();
                let imag_result = big_i | small_i;
                if real_result == 0.into() && imag_result == 0.into() {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            _ => Self {
                real: GENERAL.prefix.sa(),
                imaginary: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    pub(crate) fn aligned_xor(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite() || other.is_infinite() || (self.exploded() && other.exploded()) {
                return Self {
                    real: XOR.prefix.sa(),
                    imaginary: XOR.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.exploded() {
                if other.real.is_negative() && other.imaginary.is_negative() {
                    return !self;
                }
                if other.real.is_negative() {
                    return Self {
                        real: !self.real,
                        imaginary: self.imaginary,
                        exponent: self.exponent,
                    };
                }
                if other.imaginary.is_negative() {
                    return Self {
                        real: self.real,
                        imaginary: !self.imaginary,
                        exponent: self.exponent,
                    };
                }
                return *self;
            }
            if other.exploded() {
                if self.real.is_negative() && self.imaginary.is_negative() {
                    return !other;
                }
                if self.real.is_negative() {
                    return Self {
                        real: !other.real,
                        imaginary: other.imaginary,
                        exponent: other.exponent,
                    };
                }
                if self.imaginary.is_negative() {
                    return Self {
                        real: other.real,
                        imaginary: !other.imaginary,
                        exponent: other.exponent,
                    };
                }
                return *other;
            }
            if self.is_normal() {
                if other.real.is_negative() && other.imaginary.is_negative() {
                    return !self;
                }
                if other.real.is_negative() {
                    return Self {
                        real: !self.real,
                        imaginary: self.imaginary,
                        exponent: self.exponent,
                    };
                }
                if other.imaginary.is_negative() {
                    return Self {
                        real: self.real,
                        imaginary: !self.imaginary,
                        exponent: self.exponent,
                    };
                }
                return *self;
            }
            if other.is_normal() {
                if self.real.is_negative() && self.imaginary.is_negative() {
                    return !other;
                }
                if self.real.is_negative() {
                    return Self {
                        real: !other.real,
                        imaginary: other.imaginary,
                        exponent: other.exponent,
                    };
                }
                if self.imaginary.is_negative() {
                    return Self {
                        real: other.real,
                        imaginary: !other.imaginary,
                        exponent: other.exponent,
                    };
                }
                return *other;
            }
            return Self {
                real: XOR.prefix.sa(),
                imaginary: XOR.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            if small.real.is_negative() && small.imaginary.is_negative() {
                return !big;
            }
            if small.real.is_negative() {
                return Self {
                    real: !big.real,
                    imaginary: big.imaginary,
                    exponent: big.exponent,
                };
            }
            if small.imaginary.is_negative() {
                return Self {
                    real: big.real,
                    imaginary: !big.imaginary,
                    exponent: big.exponent,
                };
            }
            return *big;
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                if small.real.is_negative() && small.imaginary.is_negative() {
                    return !big;
                }
                if small.real.is_negative() {
                    return Self {
                        real: !big.real,
                        imaginary: big.imaginary,
                        exponent: big.exponent,
                    };
                }
                if small.imaginary.is_negative() {
                    return Self {
                        real: big.real,
                        imaginary: !big.imaginary,
                        exponent: big.exponent,
                    };
                }
                return *big;
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                if small.real.is_negative() && small.imaginary.is_negative() {
                    return !big;
                }
                if small.real.is_negative() {
                    return Self {
                        real: !big.real,
                        imaginary: big.imaginary,
                        exponent: big.exponent,
                    };
                }
                if small.imaginary.is_negative() {
                    return Self {
                        real: big.real,
                        imaginary: !big.imaginary,
                        exponent: big.exponent,
                    };
                }
                return *big;
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i16 = big.real.as_();
                big_r <<= shift;
                let small_r: i16 = small.real.as_();
                let real_result = big_r ^ small_r;
                let mut big_i: i16 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i16 = small.imaginary.as_();
                let imag_result = big_i ^ small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            16 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i32 = big.real.as_();
                big_r <<= shift;
                let small_r: i32 = small.real.as_();
                let real_result = big_r ^ small_r;
                let mut big_i: i32 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i32 = small.imaginary.as_();
                let imag_result = big_i ^ small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            32 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i64 = big.real.as_();
                big_r <<= shift;
                let small_r: i64 = small.real.as_();
                let real_result = big_r ^ small_r;
                let mut big_i: i64 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i64 = small.imaginary.as_();
                let imag_result = big_i ^ small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            64 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i128 = big.real.as_();
                big_r <<= shift;
                let small_r: i128 = small.real.as_();
                let real_result = big_r ^ small_r;
                let mut big_i: i128 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i128 = small.imaginary.as_();
                let imag_result = big_i ^ small_i;
                if real_result == 0 && imag_result == 0 {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            128 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: I256 = big.real.into();
                big_r <<= shift;
                let small_r: I256 = small.real.into();
                let real_result = big_r ^ small_r;
                let mut big_i: I256 = big.imaginary.into();
                big_i <<= shift;
                let small_i: I256 = small.imaginary.into();
                let imag_result = big_i ^ small_i;
                if real_result == 0.into() && imag_result == 0.into() {
                    return Self {
                        real: F::ZERO,
                        imaginary: F::ZERO,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let real_leading =
                    real_result.leading_ones().max(real_result.leading_zeros()) as isize;
                let imag_leading =
                    imag_result.leading_ones().max(imag_result.leading_zeros()) as isize;
                let leading = real_leading.min(imag_leading);
                let offset = small
                    .exponent
                    .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                if big.exponent.is_negative() && !offset.is_negative() {
                    return Self {
                        real: ((real_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((imag_result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self {
                    real: ((real_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    imaginary: ((imag_result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                        .as_i128()
                        .as_(),
                    exponent: offset.wrapping_add(&E::ONE),
                };
            }
            _ => Self {
                real: GENERAL.prefix.sa(),
                imaginary: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    pub(crate) fn not_circle(&self) -> Circle<F, E> {
        if self.is_undefined() {
            return *self;
        }
        return Self {
            real: !self.real,
            imaginary: !self.imaginary,
            exponent: self.exponent,
        };
    }
    pub(crate) fn circle_shl_integer(&self, shift: &E) -> Circle<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_add(shift);
        if !shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                real: self.real,
                imaginary: self.imaginary,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if shift.is_negative() && self.exponent.is_negative() && !(new_exp.wrapping_sub(&E::ONE)).is_negative()
        {
            return Self {
                real: self.real >> 1isize,
                imaginary: self.imaginary >> 1isize,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        return Self {
            real: self.real,
            imaginary: self.imaginary,
            exponent: new_exp,
        };
    }
    pub(crate) fn circle_shr_integer(&self, shift: &E) -> Circle<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_sub(shift);
        if shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                real: self.real,
                imaginary: self.imaginary,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if !shift.is_negative() && self.exponent.is_negative() && !(new_exp.wrapping_sub(&E::ONE)).is_negative()
        {
            return Self {
                real: self.real >> 1isize,
                imaginary: self.imaginary >> 1isize,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        return Self {
            real: self.real,
            imaginary: self.imaginary,
            exponent: new_exp,
        };
    }
}
