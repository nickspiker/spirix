use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    > Scalar<F, E>
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
    /// Multiplies this Scalar by another Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between two Scalars, handling special cases according to mathematical principles.
    /// For normal values, this produces the expected mathematical product. Special states follow special rules to maintain mathematical continuity even when results exceed representable ranges.
    ///
    /// Multiplication process:
    /// 0. Checks for escaped values (undefined, exploded, vanished) and applies special case handling  
    /// 1. Uses wider integer types for fraction multiplication as product lands in the high half  
    /// ```txt
    ///    □□□□□□□ ■■■■■■■ = Multiplier
    ///    □□□□□□□ ■■■■■■■ = Multiplicand
    ///         ⤪⤪⤪⤪⤪⤪      Multiply!
    ///    ■■■■■■■ □□□□□□□ = Intermediate 2x-bit product space
    ///        ↘↘↘↘↘↘↘
    ///            ■■■■■■■ = High half kept, low bits discarded to floor
    /// 2. Calculates leading Zeros/Ones to determine normalization shift and exponent nudge  
    /// 3. Adds exponents and adjusts by normalization shift (exponent_result = self.exponent + other.exponent - shift)  
    /// 4. Handles special cases where exponent exceeds MAX_EXPONENT (explode) or falls below MIN_EXPONENT (vanish)  
    /// 5. For escaped values, preserves sign and phase information while following mathematical convention  
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // Multiplying finite Scalars
    /// let eight = Scalar::<i64, i16>::from(8);
    /// let eigth = ScalarF6E4::ONE / 8;
    /// assert!(eight * eigth == 1); // Restores unity thru multiplicative inverse
    ///
    /// // Multiplying near boundaries
    /// let large = ScalarF6E4::from(64) * ScalarF6E4::MAX_NEG;
    /// let small = ScalarF6E4::from(1) / large;
    /// assert!(large * small == 1);
    ///
    /// // Multiplication preserves sign according to mathematical rule
    /// let negative = ScalarF6E4::from(-1.5);
    /// assert!((eight * negative).is_negative()); // Positive × Negative = Negative
    /// assert!((negative * negative).is_positive()); // Negative × Negative = Positive
    ///
    /// // Vanished values maintain sign thru multiplication
    /// let tiny = ScalarF6E4::MIN_POS / ScalarF6E4::from(11);
    /// assert!(tiny.vanished());
    /// let neg_tiny = tiny * ScalarF6E4::NEG_ONE;
    /// assert!(neg_tiny.vanished() && neg_tiny.is_negative());
    ///
    /// // Exploded values interact consistently with finite values
    /// let huge = ScalarF6E4::MAX * 42;
    /// assert!(huge.exploded());
    /// assert!((1 / huge).vanished()); // Inverse of exploded is vanished
    /// assert!(((-1) / huge).is_negative()); // Signs are propogated following multiplication rule
    /// assert!((huge * ScalarF6E4::NEG_ONE).exploded());
    /// assert!((huge * -1).is_negative());
    ///
    /// // Multiplying by zero always produces zero, even with escaped values
    /// assert!((huge * 0).is_zero());
    /// assert!((tiny * 0).is_zero());
    ///
    /// // Vanished × Exploded yields an undefined state (magnitude indeterminate)
    /// let undefined_product = tiny * huge;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn scalar_multiply_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite() && other.is_zero() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() && other.is_infinite() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_infinite() || other.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.exploded() && other.vanished() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.exploded() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Escaped * escaped/normal: escaped uses sign_extend, normal uses inflate
            let self_wide = self.fraction.inflate(self.is_normal());
            let other_wide = other.fraction.inflate(other.is_normal());
            let product = self_wide.w_mul(other_wide);
            let result_exploded = self.exploded() || other.exploded();
            let leading = product.leading_same();
            let n: isize = if result_exploded { 1 } else { 2 };
            let shift = leading.wrapping_sub(n);
            // Negative shift means product's top bit is above the target
            // N-level — fold into a single `shr` instead of `shl(negative)`.
            let fraction = if shift >= 0 {
                product.w_shl(shift).w_shr(Self::fraction_bits()).deflate()
            } else {
                product
                    .w_shr(Self::fraction_bits().wrapping_sub(shift))
                    .deflate()
            };
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }

        if self.fraction == Self::neg_one_normal() && other.fraction == Self::neg_one_normal() {
            let mut exp = self.exponent.wrapping_add(&other.exponent);
            if self.exponent.is_negative() && other.exponent.is_negative() && !exp.is_negative() {
                return Self {
                    fraction: Self::pos_one_vanished(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            exp = exp.wrapping_add(&E::one());
            if !self.exponent.is_negative() && !other.exponent.is_negative() && exp.is_negative() {
                return Self {
                    fraction: Self::pos_one_exploded(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: Self::pos_one_normal(),
                exponent: exp,
            };
        }

        // x86 implementation note:
        // Why this code is NOT signless
        // Spirix's fundamental multiply is signless: two's-complement inflated fractions multiplied with arithmetic shift right for floor rounding, no magnitude/sign decomposition anywhere. In the Verilog target this is exactly how it's implemented — one multiplier, one barrel shifter, no sign-separation datapath, no cmov analog.
        // Here in Rust/x86 we're forced into a hybrid because of a platform bit-width accident. Inflated fractions occupy FRAC+1 bits signed (magnitude reaches 2^FRAC at the ±1.0 boundary), so their product needs 2*FRAC+2 bits. Our `Wide` type is only 2*FRAC bits (i16 for i8 stored, i32 for i16, ..., I256 for i128 — Rust has no 2N+2-bit primitive at any width). That leaves us exactly one bit short in the worst case, and the signed multiply wraps.
        // The wrap happens to be INVISIBLE for the main-path byte extraction (shift = FRAC - leading ≤ FRAC, so the 2^W wrap correction is 0 mod 2^FRAC), which lets us keep signed arithmetic + arith shr = floor for that path. But for exploded/vanished (shift > FRAC), the wrap correction doesn't vanish mod 2^FRAC, so we reluctantly fall back to the magnitude-dance (compute |p|, logical shr, XOR a sign-flip mask). Those paths represent "too big/small to represent normally" so the magnitude precision is already lossy — rounding mode is moot there.
        // In Verilog all of this dissolves: hardware arith shr is free, register width is whatever we declare, and signed/unsigned interpretation is just wire routing. The code below is overhead paid for x86 ISA quirks, not inherent algorithmic cost.
        let p_signed = self
            .fraction
            .inflate(true)
            .w_mul(other.fraction.inflate(true));
        let fb = Self::fraction_bits();
        // v0.1 ruler: value = inflate × 2^(exp - FRAC + 1). The +1 accumulates
        // per multiplication — each operand's effective magnitude is 2× relative
        // to `inflate × 2^(exp-FRAC)`, so their product carries 4×, compensated
        // by adding 1 to the running exp.
        let sum = self.exponent.wrapping_add(&other.exponent).wrapping_add(&1u8.as_());

        // u_product = |p_signed|. LLVM compiles this conditional to `neg + cmov` (branchless at asm level) — no wider bit-ops formulation beats that.
        let expect_negative = self.is_negative() != other.is_negative();
        let u_product = if expect_negative {
            p_signed.w_neg()
        } else {
            p_signed
        };
        let leading = u_product.w_leading_zeros();

        // Exploded/vanished extract: u_product (magnitude) >> k logical, then XOR with class-appropriate sign-flip mask if neg_bit. Magnitude-based so wrap doesn't bite. Rounding mode here doesn't matter.
        let exploded_flip = Self::pos_one_exploded() ^ Self::neg_one_exploded();
        let vanished_flip = Self::pos_one_vanished() ^ Self::neg_one_vanished();
        let extract_escaped = |k: isize, flip: F| -> F {
            let pos = if k >= 0 {
                u_product.w_shr_logical(k).deflate()
            } else {
                u_product.w_shl(k.wrapping_neg()).deflate()
            };
            if expect_negative {
                pos ^ flip
            } else {
                pos
            }
        };

        // Exponent overflow (both inputs positive-exp, sum wrapped negative including AMBIGUOUS) → exploded.
        if !self.exponent.is_negative() && !other.exponent.is_negative() && sum.is_negative() {
            let k = fb.wrapping_sub(leading).wrapping_add(1);
            return Self {
                fraction: extract_escaped(k, exploded_flip),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Exponent underflow → vanished.
        if self.exponent.is_negative() && other.exponent.is_negative() && !sum.is_negative() {
            let k = fb.wrapping_sub(leading).wrapping_add(2);
            return Self {
                fraction: extract_escaped(k, vanished_flip),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Main path.
        let adj: E = (leading as isize).as_();
        let exponent = sum.wrapping_sub(&adj);
        // Tail underflow: exponent landed on AMBIGUOUS → vanished.
        if exponent == Self::ambiguous_exponent() {
            let k = fb.wrapping_sub(leading).wrapping_add(2);
            return Self {
                fraction: extract_escaped(k, vanished_flip),
                exponent,
            };
        }

        // Main path: signed arith shr of p_signed. For k <= FRAC (always true here since k_main = FRAC - leading ≤ FRAC), the wrap correction vanishes mod 2^FRAC, so the byte is correct. Arith shr = floor rounding.
        let k_main = fb.wrapping_sub(leading);
        let fraction = if k_main >= 0 {
            p_signed.w_shr(k_main).deflate()
        } else {
            p_signed.w_shl(k_main.wrapping_neg()).deflate()
        };
        // Boundary: signed extract of magnitude 2^(FRAC-1) (= POS_ONE_NORMAL) with negative result lands at the asymmetric two's complement edge. Reroute to NEG_ONE_NORMAL at exp-1 (with AMBIGUOUS check).
        if expect_negative && fraction == Self::pos_one_normal() {
            let exp_m1 = exponent.wrapping_sub(&1u8.as_());
            if exp_m1 == Self::ambiguous_exponent() {
                return Self {
                    fraction: Self::neg_one_vanished(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: Self::neg_one_normal(),
                exponent: exp_m1,
            };
        }
        Self { fraction, exponent }
    }
}
