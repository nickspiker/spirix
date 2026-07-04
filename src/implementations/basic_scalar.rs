use crate::constants::ScalarConstants;
use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

macro_rules! impl_scalar_new {
    ($($f:ty, $e:ty);*) => {
        $(
impl Scalar<$f, $e> {
    /// Creates a new Scalar from raw fraction and exponent integers.
    ///
    /// This is a low-level constructor that directly sets the internal state. For normal number creation, use `from()` which handles:
    /// - Proper normalization of the fraction
    /// - Exponent calculation
    /// - Special value mapping
    ///
    /// # Example
    /// ```
    /// # use spirix::Scalar;
    /// // 42 = binade 2^5, fraction bits 101010 in the top positions. // Stored exponent = 5 XOR i8::MIN = -123. let raw_scalar = Scalar::<i32, i8>::new(42i32 << 26, -123i8); let normal = Scalar::<i32, i8>::from(42_i8); assert!(raw_scalar == normal);
    /// ```
    #[inline]
    pub fn new(fraction: $f, exponent: $e) -> Scalar<$f, $e> {
        Scalar { fraction, exponent }
    }
}
        )*
    }
}
impl_scalar_new! {
    i8, i8;
    i16, i8;
    i32, i8;
    i64, i8;
    i128, i8;
    i8, i16;
    i16, i16;
    i32, i16;
    i64, i16;
    i128, i16;
    i8, i32;
    i16, i32;
    i32, i32;
    i64, i32;
    i128, i32;
    i8, i64;
    i16, i64;
    i32, i64;
    i64, i64;
    i128, i64;
    i8, i128;
    i16, i128;
    i32, i128;
    i64, i128;
    i128, i128
}

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
    /// Extracts the high byte (first 8 bits) from the fraction.
    ///
    /// This forms a behavioral prefix that indicates the value's classification. Normal values use the N0 convention (no MSB sign bit; sign via ~MSB); escaped and singular patterns carry their shape in the high bits and are tagged by AMBIGUOUS_EXPONENT.
    ///
    /// Singular (with AMBIGUOUS_EXPONENT) □□□□□□□□  Zero `[0]` ■■■■■■■■  Infinity `[∞]`
    ///
    /// Normal (N0, sign via ~MSB; non-AMBIGUOUS exponent) ■xxxxxxx  Positive `[+#]` □xxxxxxx  Negative `[-#]`
    ///
    /// Exploded (N1 with AMBIGUOUS_EXPONENT, explicit sign at MSB) □■xxxxxx  Positive `[+↑]` ■□xxxxxx  Negative `[-↑]`
    ///
    /// Vanished (N2 with AMBIGUOUS_EXPONENT, explicit sign at MSB) □□■xxxxx  Positive `[+↓]` ■■□xxxxx  Negative `[-↓]`
    ///
    /// Undefined (N3+ with AMBIGUOUS_EXPONENT) □□□■xxxx | ■■■□xxxx `[℘?]`
    ///
    /// The prefix is used by methods to determine the Scalar's state and behavior in operations.
    #[inline]
    pub(crate) fn prefix(&self) -> i8 {
        self.fraction.sa()
    }

    /// Checks if this Scalar is a normal number `[#]`
    ///
    /// # Description
    ///
    /// Normal numbers have a definite magnitude and participate fully in all arithmetic operations. Unlike Infinity, Zero, escaped, or undefined values, normal numbers occupy the "standard" region of numeric space where arithmetic behaves conventionally.
    ///
    /// # Returns
    ///
    /// - `[#]` ➔ `true` Normal numbers
    /// - `[0]` ➔ `false` Zero
    /// - `[∞]` ➔ `false` Infinity
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E4};
    ///
    /// // Regular values are normal let normal = Scalar::<i16, i16>::from(42); assert!(normal.is_normal());
    ///
    /// // Normal values maintain their normality thru standard operations let still_normal = normal * ScalarF4E4::PI / 2_i16; assert!(still_normal.is_normal());
    ///
    /// // Zero is not normal let zero = ScalarF4E4::ZERO; assert!(!zero.is_normal());
    ///
    /// // Infinity is not normal let infinity: ScalarF4E4 = ScalarF4E4::ONE / 0_i16; assert!(!infinity.is_normal());
    ///
    /// // Exploded values are not normal let exploded = ScalarF4E4::MAX + ScalarF4E4::MAX; assert!(!exploded.is_normal());
    ///
    /// // Vanished values are not normal let vanished: ScalarF4E4 = ScalarF4E4::MIN_POS - ScalarF4E4::MIN_POS * 1.25_f32; assert!(!vanished.is_normal());
    ///
    /// // Undefined Scalars are definitely not normal let undefined: ScalarF4E4 = zero / 0_i16; assert!(!undefined.is_normal());
    ///
    /// // Operations that exceed representable range escape normality let no_longer_normal = ScalarF4E4::MAX_NEG.square(); assert!(!no_longer_normal.is_normal());
    /// ```
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.exponent != Self::ambiguous_exponent()
    }

    /// Checks if this Scalar is undefined `[℘?]`
    ///
    /// # Description
    ///
    /// Undefined Scalars represent operations that have no known or agreed upon mathematical result. Spirix uses specific bit patterns to track various types of undefined states, maintaining "first cause" information.
    ///
    /// Undefined states propagate thru operations, preserving the original undefined operation while allowing computation to continue.
    ///
    /// # Returns
    ///
    /// - `[℘?]` ➔ `true` Any undefined state
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[0]` ➔ `false` Zero
    /// - `[∞]` ➔ `false` Infinity
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Zero over Zero creates an undefined state let undefined: Scalar::<i32, i8> = Scalar::<i32, i8>::ZERO / 0_i32; assert!(undefined.is_undefined());
    ///
    /// // Normal numbers are defined let normal = ScalarF5E3::from(42_i32); assert!(!normal.is_undefined());
    ///
    /// // Zero is defined let zero = ScalarF5E3::ZERO; assert!(!zero.is_undefined());
    ///
    /// // Escaped values are defined let exploded = ScalarF5E3::MAX * ScalarF5E3::MIN; assert!(!exploded.is_undefined());
    ///
    /// // Exploded values in certain operations stay defined let vanished: ScalarF5E3 = 1_i32 / exploded; assert!(!vanished.is_undefined());
    ///
    /// // But some operations produce undefined results let undefined_exploded_add: ScalarF5E3 = exploded + 1_i32; assert!(undefined_exploded_add.is_undefined());
    ///
    /// // Wheras others do not let one: ScalarF5E3 = vanished + 1_i32; assert!(one == 1_i32);
    ///
    /// // Infinity is defined let infinity: ScalarF5E3 = normal / 0_i32; assert!(!infinity.is_undefined());
    /// ```
    #[inline]
    /// XOR fingerprint of the prefix: result bit i = (prefix bit i) XOR (prefix bit i-1). Encodes all four escape-class patterns at once, so each `is_*` check below is a branchless mask. Unsigned shift to skip Rust's `i8::MIN << 1` overflow check. 0 or 1   → uniform (prefix == 0 or -1, i.e. ZERO or INFINITY) [2, 63]  → undefined (top 3 same, lower bits not — bit 7 & 6 of XOR clear, but not all-zero) [64, 127] → vanished (top 2 same, third differs — bit 7 of XOR clear, bit 6 set) [128, 255] → exploded (top 2 differ — bit 7 of XOR set)
    #[inline]
    fn class_xor(&self) -> u8 {
        let p = self.prefix() as u8;
        p ^ (p << 1)
    }

    pub fn is_undefined(&self) -> bool {
        if self.is_normal() {
            return false;
        }
        let x = self.class_xor();
        (x & 0xC0) == 0 && x >= 2
    }

    pub(crate) fn is_uniform(&self) -> bool {
        !self.is_normal() && self.class_xor() < 2
    }

    pub fn is_exploded(&self) -> bool {
        !self.is_normal() && self.class_xor() & 0x80 != 0
    }

    pub fn is_vanished(&self) -> bool {
        !self.is_normal() && (self.class_xor() & 0xC0) == 0x40
    }

    /// Checks if this Scalar's magnitude is negligible `[0]`, `[↓]`
    ///
    /// # Description
    ///
    /// Negligible values have effectively zero magnitude. This includes both actual Zero `[0]` and vanished values `[↓]` that have become so small they no longer meaningfully contribute to addition or subtraction.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[↓]` ➔ `true` Vanished values
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E5};
    ///
    /// // Zero: The original negligible number let zero = Scalar::<i16, i32>::ZERO; assert!(zero.is_negligible());
    ///
    /// // A number so small it's effectively zero let vanished: ScalarF4E5 = ScalarF4E5::MIN_POS / 57_i16; assert!(vanished.is_negligible());
    ///
    /// // Even tiny numbers maintain their sign let neg_vanished: ScalarF4E5 = vanished * -1_i16; assert!(neg_vanished.is_negative());
    ///
    /// // Normal values are not negligible let meaning = ScalarF4E5::from(1729_i16); assert!(!meaning.is_negligible());
    ///
    /// // Large escaped values are not negligible let exploded = ScalarF4E5::MAX * ScalarF4E5::MIN; assert!(!exploded.is_negligible());
    ///
    /// // Undefined Scalars are not negligible let undefined: ScalarF4E5 = ScalarF4E5::ZERO / 0_i16; assert!(!undefined.is_negligible());
    ///
    /// // Infinite Scalars are not negligible let infinite: ScalarF4E5 = ScalarF4E5::ONE / 0_i16; assert!(!infinite.is_negligible());
    ///
    /// // Reciprocals of Infinite Scalars are Zero! let inf_recip: ScalarF4E5 = 1_i16 / infinite; assert!(inf_recip.is_negligible()); assert!(inf_recip == 0_i16);
    ///
    /// // Division by a vanished value creates an exploded result let also_exploded: ScalarF4E5 = 1_i16 / vanished; assert!(!also_exploded.is_negligible());
    /// ```
    #[inline]
    pub fn is_negligible(&self) -> bool {
        // Negligible = Zero or Vanished.
        // Do NOT shortcut on `prefix == 0`: the negative boundary values (-1, -2, -4, … = -2^k) also store a zero fraction, but with a NORMAL exponent, so a prefix-only test wrongly flags them as negligible.
        // is_zero() checks the ambiguous exponent too, which is what distinguishes Zero from -2^k.
        self.is_zero() || self.is_vanished()
    }

    /// Returns true if this Scalar is an infinitesimal value `[↓]` (close but not equal to Zero)
    ///
    /// # Description
    ///
    /// Vanished values are numbers that have become so small their magnitude is effectively zero, but they retain their sign information and can participate in multiplication and division, but are treated as Zero in addition and subtraction.
    ///
    /// # Returns
    ///
    /// - `[↓]` ➔ `true` Vanished values
    /// - `[0]` ➔ `false` Zero
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E3};
    ///
    /// // Create a ridiculously small positive number let vanished: Scalar::<i16, i8> = Scalar::<i16, i8>::MIN_POS / 123.45_f32; assert!(vanished.vanished()); assert!(vanished.is_positive());
    ///
    /// // And a ridiculously small negative number let vanished_negative: ScalarF4E3 = ScalarF4E3::MIN_POS / -12_i16; assert!(vanished_negative.vanished()); assert!(vanished_negative.is_negative());
    ///
    /// // Actual Zero is not vanished - it's truly Zero let actual_zero = ScalarF4E3::ZERO; assert!(!actual_zero.vanished());
    ///
    /// // Normal values are not vanished let normal = ScalarF4E3::from(42_i16); assert!(!normal.vanished());
    ///
    /// // Large escaped values aren't vanished - they're exploded! let ginormous = ScalarF4E3::MAX.pow(ScalarF4E3::MAX); assert!(!ginormous.vanished()); assert!(ginormous.exploded());
    ///
    /// // Normal division by a vanished value produces an exploded result let exploded: ScalarF4E3 = 1_i16 / vanished; assert!(exploded.exploded()); assert!(!exploded.vanished());
    ///
    /// // Reciprocal of Infinity is not vanished, it's Zero! let infinity: ScalarF4E3 = ScalarF4E3::from(42_i16) / 0_i16; let zero: ScalarF4E3 = 1_i16 / infinity; assert!(!zero.vanished()); assert!(zero.is_zero());
    /// ```
    #[inline]
    pub fn vanished(&self) -> bool {
        self.is_vanished()
    }

    /// Returns true if this Scalar is ridiculously large `[↑]` but not ∞
    ///
    /// # Description
    ///
    /// Exploded values are numbers that have grown so large their magnitude can no longer be recorded, but they maintain their sign information. They participate meaningfully in absolute operations like multiplication and division operations, but relative operations like addition and subtraction with exploded or infinite Scalars will produce undefined results.
    ///
    /// # Returns
    ///
    /// - `[↑]` ➔ `true` Exploded values
    /// - `[0]` ➔ `false` Zero
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Create an astronomically large positive Scalar let biggin: ScalarF5E3 = ScalarF5E3::MAX * ScalarF5E3::MAX; assert!(biggin.exploded()); assert!(biggin.is_positive()); // Scalars don't saturate to infinity assert!(!biggin.is_infinite());
    ///
    /// // Create an exploded negative Scalar let huge_negative: ScalarF5E3 = biggin * -1_i32; assert!(huge_negative.exploded()); assert!(huge_negative.is_negative());
    ///
    /// // Actual Zero is not vanished let actual_zero = ScalarF5E3::ZERO; assert!(!actual_zero.vanished());
    ///
    /// // Infinity is not exploded either let infinity: ScalarF5E3 = 1_i32 / actual_zero; assert!(!infinity.exploded());
    ///
    /// // Normal values are, well, normal! let normal = ScalarF5E3::from(42_i32); assert!(normal.is_normal());
    ///
    /// // Small escaped values aren't Zero - they're vanished! let vanished = ScalarF5E3::MIN_POS.square(); assert!(!vanished.is_zero()); assert!(vanished.vanished());
    ///
    /// // Multiplying or dividing exploded Scalars maintains the exploded state let still_exploded: ScalarF5E3 = biggin / 42_i32; assert!(still_exploded.exploded()); let definitely_exploded: ScalarF5E3 = biggin * 42_i32; assert!(definitely_exploded.exploded());
    ///
    /// // Division by vanished values produces exploded results let tiny = ScalarF5E3::MIN_POS.square(); let also_exploded: ScalarF5E3 = 1_i32 / tiny; assert!(also_exploded.exploded());
    /// ```
    #[inline]
    pub fn exploded(&self) -> bool {
        self.exponent == Self::ambiguous_exponent() && self.is_exploded()
    }

    /// Returns true if this Scalar is beyond normal magnitude `[↑]` or `[∞]`
    ///
    /// # Description
    ///
    /// Transfinite values are numbers that have grown beyond representable magnitude. This includes both directional exploded values `[↑]` that maintain phase information, and mathematical infinity `[∞]` which represents a singularity, like division by zero.
    ///
    /// # Returns
    ///
    /// - `[↑]` ➔ `true` Exploded values
    /// - `[∞]` ➔ `true` Infinity
    /// - `[0]` ➔ `false` Zero
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E4};
    ///
    /// // Exploded Scalars are transfinite let exploded = ScalarF5E4::MAX * ScalarF5E4::MAX; assert!(exploded.is_transfinite()); assert!(exploded.exploded()); assert!(!exploded.is_infinite()); // But it's not infinity
    ///
    /// // Exploded negatives are also transfinite let neg_exploded: ScalarF5E4 = exploded * -1_i32; assert!(neg_exploded.is_transfinite()); assert!(neg_exploded.is_negative());
    ///
    /// // Division by zero produces true mathematical infinity let infinity: ScalarF5E4 = ScalarF5E4::ONE / 0_i32; assert!(infinity.is_transfinite()); assert!(!infinity.exploded()); // Not the same as exploded!
    ///
    /// // Normal values are not transfinite let normal = ScalarF5E4::from(42_i32); assert!(!normal.is_transfinite());
    ///
    /// // Zero is not transfinite let zero = ScalarF5E4::ZERO; assert!(!zero.is_transfinite());
    ///
    /// // Vanished values are not transfinite (they're the opposite!) let tiny = ScalarF5E4::MAX_NEG.square(); assert!(!tiny.is_transfinite());
    ///
    /// // The reciprocal of transfinite is tiny let vanished: ScalarF5E4 = 1_i32 / exploded; let also_zero: ScalarF5E4 = 1_i32 / infinity; assert!(vanished.is_negligible()); assert!(also_zero.is_negligible());
    ///
    /// // Anything over infinity is exactly Zero let zero_again = exploded / infinity; assert!(zero_again.is_zero()); // Except for infinity over infinity, of course let undefined = ScalarF5E4::INFINITY / infinity; assert!(undefined.is_undefined());
    ///
    /// // Math operations with infinity follow mathematical rules let also_infinity = infinity * ScalarF5E4::PI; assert!(also_infinity.is_transfinite()); let still_infinity = infinity.pow(ScalarF5E4::E); assert!(still_infinity.is_transfinite());
    /// ```
    pub fn is_transfinite(&self) -> bool {
        self.exponent == Self::ambiguous_exponent()
            && (self.is_exploded() || self.fraction == (-F::one()))
    }

    /// Returns true if this Scalar represents a finite number `[0]`, `[#]`
    ///
    /// # Description
    ///
    /// Finite numbers have a known magnitude and can participate fully in all arithmetic operations. This includes all normal numbers and Zero, but excludes escaped values (exploded and vanished) and undefined states.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[#]` ➔ `true` Normal numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E4};
    ///
    /// // Normal values are finite let normal = Scalar::<i32, i16>::from(42_i32); assert!(normal.is_finite());
    ///
    /// // Zero is finite let zero = normal - normal; assert!(zero.is_finite());
    ///
    /// // Pi is finite let pi = ScalarF5E4::PI; assert!(pi.is_finite());
    ///
    /// // Exploded values are not finite let exploded: ScalarF5E4 = ScalarF5E4::MAX * 2_i32; assert!(!exploded.is_finite());
    ///
    /// // Vanished values are not finite let vanished: ScalarF5E4 = ScalarF5E4::MIN_POS / 28_i32; assert!(!vanished.is_finite());
    ///
    /// // Undefined Scalars are not finite let infinity_f: ScalarF5E4 = ScalarF5E4::ONE / 0_i32; let undefined: ScalarF5E4 = infinity_f * ScalarF5E4::ZERO; assert!(!undefined.is_finite());
    ///
    /// // Operations that produce normal results usually return finite values let still_finite: ScalarF5E4 = normal + 1_i32; assert!(still_finite.is_finite());
    ///
    /// // Operations that exceed representable range escape finiteness let no_longer_finite: ScalarF5E4 = ScalarF5E4::MAX + ScalarF5E4::MAX / 2_i32; assert!(!no_longer_finite.is_finite());
    ///
    /// // Infinity is definitely not finite! let infinity: ScalarF5E4 = ScalarF5E4::MAX / 0_i32; assert!(!infinity.is_finite());
    /// ```
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.is_normal() || self.is_zero()
    }

    /// Returns true if this Scalar is mathematical Zero `[0]`
    ///
    /// # Description
    ///
    /// Zero represents the additive identity in mathematics. It has a unique bit pattern with all fraction bits clear (00000000) and ambiguous exponent, creating symmetry with Infinity (11111111).
    ///
    /// Unlike vanished values which approach but never equal Zero, this is the real deal Zero that follows standard rules (multiplication by Zero yields Zero, division by Zero produces infinity).
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // The one and only Zero let zero = Scalar::<i64, i16>::ZERO; assert!(zero.is_zero());
    ///
    /// // Even a very small number is not Zero let tiny: ScalarF6E4 = ScalarF6E4::MIN_POS / 61_i64; assert!(!tiny.is_zero()); assert!(tiny.vanished());  // It's vanished, not Zero
    ///
    /// // Normal values are not Zero let normal = ScalarF6E4::from(42_i64); assert!(!normal.is_zero());
    ///
    /// // Exploded values are not Zero let huge = ScalarF6E4::MAX * ScalarF6E4::MAX; assert!(!huge.is_zero());
    ///
    /// // Infinity is not Zero (it's the opposite!) let infinity: ScalarF6E4 = ScalarF6E4::ONE / 0_i64; assert!(!infinity.is_zero());
    ///
    /// // Multiplication by Zero always yields Zero let still_zero = zero * ScalarF6E4::PI; assert!(still_zero.is_zero()); // Except with Infinity let undefined = zero * ScalarF6E4::INFINITY; assert!(!undefined.is_zero());
    ///
    /// // Reciprocal of infinity is Zero let also_zero = ScalarF6E4::ONE / infinity; assert!(also_zero.is_zero());
    ///
    /// // Adding a normal Scalar to Zero or vanished gives a normal Scalar let normal_again: ScalarF6E4 = zero + 163_i64 + tiny + also_zero; assert!(!normal_again.is_zero()); assert!(normal_again.is_normal()); assert!(normal_again == 163_i64);
    /// ```
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.exponent == Self::ambiguous_exponent() && self.fraction == F::zero()
    }

    /// Returns true if this Scalar is mathematical infinity `[∞]`
    ///
    /// # Description
    ///
    /// Infinity represents the result of operations like division by zero, tangent of π/2, or other mathematical concepts that produce a singularity. Unlike exploded values which maintain sign information, infinity is signless and represents a true mathematical concept rather than a computational limitation.
    ///
    /// # Returns
    ///
    /// - `[∞]` ➔ `true` Infinity
    /// - `[0]` ➔ `false` Zero
    /// - `[#]` ➔ `false` Normal numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Division by zero produces infinity let infinity: ScalarF5E3 = ScalarF5E3::ONE / 0_i32; assert!(infinity.is_infinite());
    ///
    /// // Exploded values are not infinity let exploded = ScalarF5E3::MAX * ScalarF5E3::MAX; assert!(!exploded.is_infinite()); assert!(exploded.exploded());
    ///
    /// // Both are transfinite assert!(infinity.is_transfinite()); assert!(exploded.is_transfinite());
    ///
    /// // Normal values are not infinity let normal = ScalarF5E3::from(42_i32); assert!(!normal.is_infinite());
    ///
    /// // Zero is not infinity (it's the opposite!) let zero = ScalarF5E3::ZERO; assert!(!zero.is_infinite());
    ///
    /// // Reciprocal of infinity is exactly zero let zero_again = ScalarF5E3::ONE / infinity; assert!(zero_again.is_zero());
    ///
    /// // Infinity multiplied by any non-zero value remains infinity let still_infinity = infinity * ScalarF5E3::PI; assert!(still_infinity.is_infinite());
    ///
    /// // Infinity divided by any non-zero value remains infinity let still_infinity: ScalarF5E3 = infinity / 1000_i32; assert!(still_infinity.is_infinite());
    ///
    /// // infinity times zero is undefined let undefined: ScalarF5E3 = infinity * ScalarF5E3::ZERO; assert!(!undefined.is_infinite()); assert!(undefined.is_undefined());
    /// ```
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.exponent == Self::ambiguous_exponent() && self.fraction == (-F::one())
    }

    /// Returns true if this Scalar is positive `[+#]`, `[+↑]`, `[+↓]`
    ///
    /// # Description
    ///
    /// Tests if this Scalar has a positive value. Works for normal numbers, exploded positive values, and vanished positive values.
    ///
    /// # Returns
    ///
    /// - `[+#]` ➔ `true` Positive normal numbers
    /// - `[+↑]` ➔ `true` Positive exploded values
    /// - `[+↓]` ➔ `true` Positive vanished values
    /// - `[0]` ➔ `false` Zero
    /// - `[-#]` ➔ `false` Negative normal numbers
    /// - `[-↑]` ➔ `false` Negative exploded values
    /// - `[-↓]` ➔ `false` Negative vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E5};
    ///
    /// // Normal positive numbers let positive = Scalar::<i64, i32>::from(42); assert!(positive.is_positive());
    ///
    /// // Zero is not positive let zero = ScalarF6E5::ZERO; assert!(!zero.is_positive());
    ///
    /// // Negative numbers are not positive let negative = ScalarF6E5::from(-37); assert!(!negative.is_positive());
    ///
    /// // Positive vanished values are positive let tiny: ScalarF6E5 = ScalarF6E5::MIN_POS / 131071_i64; assert!(tiny.is_positive());
    ///
    /// // Positive exploded values are positive let huge: ScalarF6E5 = positive.pow(87539319_i64); assert!(huge.is_positive());
    ///
    /// // Infinities aren't positive let infinity: ScalarF6E5 = ScalarF6E5::TAU / 0_i64; assert!(!infinity.is_positive());
    ///
    /// // Neither are undefined Scalars let undefined: ScalarF6E5 = ScalarF6E5::ZERO / 0_i64; assert!(!undefined.is_positive());
    /// ```
    #[inline]
    pub fn is_positive(&self) -> bool {
        // For normal values: sign is ~stored[MSB], so positive when stored MSB=1 (negative stored)
        if self.is_normal() {
            return self.fraction.is_negative();
        }

        // For ambiguous states: sign is stored directly in the bit pattern
        let prefix = self.prefix();
        let top_three = prefix >> 5;
        if top_three == top_three.rotate_right(1) {
            return false; // Zero, Infinity, or undefined
        }
        !top_three.is_negative()
    }

    /// Returns true if this Scalar is negative `[-#]`, `[-↑]`, `[-↓]`
    ///
    /// # Description
    ///
    /// Tests if this Scalar has a negative value. Works for normal numbers, exploded negative values, and vanished negative values.
    ///
    /// # Returns
    ///
    /// - `[-#]` ➔ `true` Negative normal numbers
    /// - `[-↑]` ➔ `true` Negative exploded values
    /// - `[-↓]` ➔ `true` Negative vanished values
    /// - `[0]` ➔ `false` Zero
    /// - `[+#]` ➔ `false` Positive normal numbers
    /// - `[+↑]` ➔ `false` Positive exploded values
    /// - `[+↓]` ➔ `false` Positive vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E5};
    ///
    /// // Normal negative numbers let negative = Scalar::<i64, i32>::from(-42_i64); assert!(negative.is_negative());
    ///
    /// // Zero is not negative let zero = ScalarF6E5::ZERO; assert!(!zero.is_negative());
    ///
    /// // Positive numbers are not negative let positive = ScalarF6E5::from(37_i64); assert!(!positive.is_negative());
    ///
    /// // Negative vanished values are negative let tiny: ScalarF6E5 = ScalarF6E5::MIN_POS / -131071_i64; assert!(tiny.is_negative());
    ///
    /// // Negative exploded values are negative let huge: ScalarF6E5 = ScalarF6E5::MIN * 2_i64; assert!(huge.is_negative());
    ///
    /// // Infinities aren't negative let infinity: ScalarF6E5 = ScalarF6E5::TAU / 0_i64; assert!(!infinity.is_negative());
    ///
    /// // Neither are undefined Scalars let undefined: ScalarF6E5 = ScalarF6E5::ZERO / 0_i64; assert!(!undefined.is_negative());
    /// ```
    #[inline]
    pub fn is_negative(&self) -> bool {
        // For normal values: sign is ~stored[MSB], so negative when stored MSB=0 (non-negative stored)
        if self.is_normal() {
            return !self.fraction.is_negative();
        }

        // For ambiguous states: sign is stored directly in the bit pattern
        let prefix = self.prefix();
        let top_three = prefix >> 5;
        if top_three == top_three.rotate_right(1) {
            return false; // Zero, Infinity, or undefined
        }
        top_three.is_negative()
    }

    /// Returns true if this Scalar represents an integer `[#.0]`, `[↑]`, `[0]`
    ///
    /// # Description
    ///
    /// Tests if this Scalar has no numeric fractional portion. This includes normal whole number values, exploded values, Zero, and Infinity.
    ///
    /// # Returns
    ///
    /// - `[#.0]` ➔ `true` Normal integers
    /// - `[↑]` ➔ `true` Exploded values
    /// - `[0]` ➔ `true` Zero
    /// - `[∞]` ➔ `false` Infinity
    /// - `[#.#]` ➔ `false` Normal values with fractional parts
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF3E4};
    ///
    /// // The answer to the meaning of life is an integer let integer = Scalar::<i8, i16>::from(42_i8); assert!(integer.is_integer());
    ///
    /// // Zero is an integer let zero = ScalarF3E4::ZERO; assert!(zero.is_integer());
    ///
    /// // Negative integers are still integers let neg_int = ScalarF3E4::from(-7_i8); assert!(neg_int.is_integer());
    ///
    /// // Numbers with fractional values are not integers let fract = ScalarF3E4::from(1.122757_f64); assert!(!fract.is_integer());
    ///
    /// // Exploded values are considered integers let huge = ScalarF3E4::MAX.pow(15_i8); assert!(huge.is_integer());
    ///
    /// // Infinity is not an integer let infinity: ScalarF3E4 = ScalarF3E4::ONE / 0_i8; assert!(!infinity.is_integer());
    ///
    /// // Vanished values are not integers let tiny: ScalarF3E4 = ScalarF3E4::MIN_POS / 99_i8; assert!(!tiny.is_integer());
    ///
    /// // Undefined states are not integers let undefined: ScalarF3E4 = ScalarF3E4::ZERO / 0_i8; assert!(!undefined.is_integer());
    /// ```
    #[inline]
    pub fn is_integer(&self) -> bool {
        // AMBIG=0: handle the AMBIG sentinel first (it's the only stored value where the value is non-normal).
        if self.exponent == Self::ambiguous_exponent() {
            if self.fraction == F::zero() {
                return true;
            } // Zero
            return self.is_exploded(); // Exploded values are integers
        }
        // Recover logical k: stored ^ binade_origin.
        let logical_k_e: E = self.exponent ^ Self::binade_origin();
        let logical_k: isize = logical_k_e.saturate();
        let frac_bits = Self::fraction_bits();
        if logical_k >= frac_bits.wrapping_sub(1) {
            // Magnitude bit at FRAC-1 ⇒ logical_k ≥ FRAC-1 means every representable value at this binade is an integer (no fractional sub-bits remain below).
            return true;
        }
        if logical_k == -1 {
            // k=-1 binade covers magnitude in [0.5, 1). The only integer in that range is the boundary value v = -2.0 (stored fraction = 0, decoding to -1.0 once the k=-1 scale is applied). Positive values in [+0.5, +1) and other negatives in (-1, -0.5) are non-integers.
            return self.fraction == F::zero();
        }
        if logical_k < 0 {
            return false;
        }
        // For 0 ≤ k < FRAC-1: low (FRAC-1-k) bits of the stored fraction must all be zero (they represent the value's fractional sub-bits). Equivalent to `fraction << (k+1) == 0`.
        (self.fraction << logical_k.wrapping_add(1)) == F::zero()
    }

    /// Returns true if this value is a valid integer within the contiguous integer range
    ///
    /// # Description
    ///
    /// Tests if this Scalar represents an integer that falls within the contiguous range where both (n+1) and (n-1) can also be represented exactly. As integer values grow larger, the gaps between representable values increase, eventually reaching a point where consecutive integers can't be represented.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[#.0]` ➔ `true` Integers within contiguous range
    /// - `[##]` ➔ `false` Integers outside contiguous range
    /// - `[#.#]` ➔ `false` Values with fractional parts
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E3};
    ///
    /// // Integers within contiguous range let small_int = ScalarF4E3::from(42_i16); assert!(small_int.is_contiguous());
    ///
    /// // Zero is within the contiguous range let zero = ScalarF4E3::ZERO; assert!(zero.is_contiguous());
    ///
    /// // Large integers outside contiguous range return false, // even tho they are precisely representable let large_int: ScalarF4E3 = ScalarF4E3::MAX_CONTIGUOUS * 2_i16; assert!(!large_int.is_contiguous());
    ///
    /// // Fractional values are not contiguous integers let fract = ScalarF4E3::PI; assert!(!fract.is_contiguous());
    ///
    /// // Vanished values and infinite states are never contiguous let tiny = ScalarF4E3::MIN_POS / ScalarF4E3::MAX; assert!(!tiny.is_contiguous()); let infinity: ScalarF4E3 = ScalarF4E3::ONE / 0_i16; assert!(!infinity.is_contiguous());
    /// ```
    #[inline]
    pub fn is_contiguous(&self) -> bool {
        // First check if it's an integer at all
        if !self.is_integer() {
            return false;
        }

        // Zero is always contiguous
        if self.is_zero() {
            return true;
        }

        // For normal numbers with logical k ≥ 0 (magnitude ≥ 1.0)
        if self.is_normal() {
            let logical_k_e: E = self.exponent ^ Self::binade_origin();
            let logical_k: isize = logical_k_e.saturate();
            if logical_k < 0 {
                return false; // magnitude < 1, not a representable non-zero integer
            }
            // logical k >= FRAC: all stored bits are integer part — beyond contiguous range.
            if logical_k >= Self::fraction_bits() {
                return false;
            }
            // For contiguous integers, we need the value to be representable exactly. This is already guaranteed by is_integer(), so we just need to check if it's within the contiguous range.
            return true;
        }

        // Handle exploded integer patterns (at AMBIG exponent).
        if self.exponent == Self::ambiguous_exponent() {
            let prefix = self.prefix();
            let top_two = prefix >> 6;
            return top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8;
        }

        false
    }

    /// Negates this Scalar in place. Default fast path.
    ///
    /// Handles all cases by enumerated equality checks at boundaries, plain `wrapping_neg` elsewhere. On a CPU with good branch prediction this runs in ~3 cycles for the common non-boundary normal case — ~4-5× faster than the unified pipeline variant. Negation is rare enough in profiles that the constant-time-ish unified variant's uniformity benefits don't outweigh the cycle count for software hot paths.
    ///
    /// Three classes of values, three behaviors:
    ///
    /// Signless (zero, infinity, undefined): no-op.
    ///
    /// Escaped (exploded, vanished): canonical swap between the two poles at each N-level.
    ///
    /// Normal: `wrapping_neg` on the stored fraction, with pos_one ↔ neg_one boundaries handled by shifting the exponent.
    ///
    /// Produces the same result as [`Self::scalar_negate_unified`] — see that method for when the unified pipeline variant is preferred (FPGA targets, side-channel sensitivity, patent spec consistency).
    pub(crate) fn scalar_negate(&mut self) {
        if !self.is_normal() {
            let top_three = self.prefix() >> 5;
            if top_three == top_three.rotate_right(1) {
                return; // signless: zero, infinity, undefined
            }
            if self.fraction == Self::pos_one_exploded() {
                self.fraction = Self::neg_one_exploded();
            } else if self.fraction == Self::neg_one_exploded() {
                self.fraction = Self::pos_one_exploded();
            } else if self.fraction == Self::pos_one_vanished() {
                self.fraction = Self::neg_one_vanished();
            } else if self.fraction == Self::neg_one_vanished() {
                self.fraction = Self::pos_one_vanished();
            } else {
                self.fraction = self.fraction.wrapping_neg();
            }
            return;
        }
        if self.fraction == Self::pos_one_normal() {
            self.exponent = self.exponent.wrapping_sub(&E::one());
            if self.exponent == Self::ambiguous_exponent() {
                self.fraction = Self::neg_one_vanished();
            } else {
                self.fraction = Self::neg_one_normal();
            }
        } else if self.fraction == Self::neg_one_normal() {
            self.exponent = self.exponent.wrapping_add(&E::one());
            if self.exponent == Self::ambiguous_exponent() {
                self.fraction = Self::pos_one_exploded();
            } else {
                self.fraction = Self::pos_one_normal();
            }
        } else {
            self.fraction = self.fraction.wrapping_neg();
        }
    }

    /// Negates this Scalar in place using the unified inflate→negate→normalize pipeline shared with add/sub/mul/div. Branch-free except for one unavoidable signless check (zero/infinity/undefined have no sign and must pass thru unchanged).
    ///
    /// Steps:
    /// 1. Inflate stored fraction to the wide signed representation.
    /// 2. `wrapping_neg` the inflated value.
    /// 3. Re-normalize: count leading same bits, shift so the result lands at the class-appropriate normalization level (FRAC for normal, N-1 for exploded, N-2 for vanished), adjust exponent for normal, deflate back to stored form.
    ///
    /// ~5-7× more instructions than [`Self::scalar_negate`] on x86, but near- constant timing (one `is_normal` branch, no input-dependent data misprediction). Preferred when:
    /// - targeting the FPGA as reference (inflate pipeline is already in hw);
    /// - side-channel resistance matters;
    /// - uniformity with the patent's no-special-case narrative matters.
    pub fn scalar_negate_unified(&mut self) {
        if !self.is_normal() {
            let top_three = self.prefix() >> 5;
            if top_three == top_three.rotate_right(1) {
                return;
            }
        }

        let was_normal = self.is_normal();
        let target: isize = if was_normal {
            Self::fraction_bits()
        } else if self.vanished() {
            2
        } else {
            1
        };

        let wide = self.fraction.inflate(was_normal).w_neg();
        if wide.w_is_zero() {
            self.fraction = F::zero();
            self.exponent = Self::ambiguous_exponent();
            return;
        }

        let leading = wide.leading_same();
        let fb = Self::fraction_bits();
        let shl = leading.wrapping_sub(target);
        let shifted = if shl >= 0 {
            wide.w_shl(shl)
        } else {
            wide.w_shr(shl.wrapping_neg())
        };

        if was_normal {
            self.fraction = shifted.deflate();
            let exp_adj: E = shl.as_();
            let new_exp = self.exponent.wrapping_sub(&exp_adj);
            if new_exp == Self::ambiguous_exponent() {
                let esc_target: isize = if shl >= 0 { 2 } else { 1 };
                let esc_shl = leading.wrapping_sub(esc_target);
                let esc_shifted = if esc_shl >= 0 {
                    wide.w_shl(esc_shl)
                } else {
                    wide.w_shr(esc_shl.wrapping_neg())
                };
                self.fraction = esc_shifted.w_shr(fb).deflate();
                self.exponent = Self::ambiguous_exponent();
            } else {
                self.exponent = new_exp;
            }
        } else {
            self.fraction = shifted.w_shr(fb).deflate();
            self.exponent = Self::ambiguous_exponent();
        }
    }

    /// Returns the magnitude of this Scalar
    ///
    /// # Description
    ///
    /// Computes the magnitude (absolute value) by determining the distance from Zero, preserving the scale but removing the sign. For negative values, this returns an equivalent positive value.
    ///
    /// # Returns
    ///
    /// - `[+#]` ➔ `[+#]` Positive value unchanged
    /// - `[-#]` ➔ `[+#]` Equivalent positive value
    /// - `[+↑]` ➔ `[+↑]` Positive exploded unchanged
    /// - `[-↑]` ➔ `[+↑]` Positive exploded equivalent
    /// - `[+↓]` ➔ `[+↓]` Positive vanished unchanged
    /// - `[-↓]` ➔ `[+↓]` Positive vanished equivalent
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` Undefined state unchanged
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E6};
    ///
    /// // Magnitude of negative is positive let negative = Scalar::<i16, i64>::from(-42); assert!(negative.magnitude() == ScalarF4E6::from(42));
    ///
    /// // Magnitude of positive remains the same let positive = ScalarF4E6::from(123); assert!(positive.magnitude() == positive);
    ///
    /// // Magnitude of Zero is Zero let zero = ScalarF4E6::ZERO; assert!(zero.magnitude() == zero);
    ///
    /// // Magnitude preserves scale of vanished values let tiny_neg: ScalarF4E6 = ScalarF4E6::MIN_POS / -28_i16; assert!(tiny_neg.magnitude().is_positive()); assert!(tiny_neg.magnitude().vanished());
    ///
    /// // Magnitude preserves scale of exploded values let huge_neg = ScalarF4E6::MIN * ScalarF4E6::MAX; assert!(huge_neg.magnitude().is_positive()); assert!(huge_neg.magnitude().exploded());
    ///
    /// // Undefined states pass thru magnitude() unchanged let undefined: ScalarF4E6 = ScalarF4E6::ZERO / 0_i16; assert!(undefined.magnitude().is_undefined());
    /// ```
    pub fn magnitude(&self) -> Self {
        if self.is_positive() || self.is_zero() || self.is_infinite() || self.is_undefined() {
            return *self;
        }
        -self
    }

    /// Returns the normalized unit form of this Scalar `[+1]` or `[-1]`
    ///
    /// # Description
    ///
    /// Creates a unit Scalar (magnitude 1) with the same sign as this Scalar. This preserves the orientation while normalizing the magnitude.
    ///
    /// # Returns
    ///
    /// - `[+#]` ➔ `[+1]` Positive unit value
    /// - `[-#]` ➔ `[-1]` Negative unit value
    /// - `[+↑]` ➔ `[+1]` Positive unit value
    /// - `[-↑]` ➔ `[-1]` Negative unit value
    /// - `[+↓]` ➔ `[+1]` Positive unit value
    /// - `[-↓]` ➔ `[-1]` Negative unit value
    /// - `[0]` ➔ `[℘±∅]` Sign indeterminate undefined state
    /// - `[∞]` ➔ `[℘±∅]` Sign indeterminate undefined state
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Sign of a positive number is 1 let positive = Scalar::<i32, i8>::from(42); assert!(positive.sign() == 1);
    ///
    /// // Sign of a negative number is -1 let negative = ScalarF5E3::from(-3.14); assert!(negative.sign() == -1);
    ///
    /// // Sign of Zero is undefined (no direction) let zero = ScalarF5E3::ZERO; assert!(zero.sign().is_undefined());
    ///
    /// // Sign of Infinity is undefined (no direction) let infinity: ScalarF5E3 = ScalarF5E3::ONE / 0_i32; assert!(infinity.sign().is_undefined());
    ///
    /// // Sign of undefined remains unchanged let undefined: ScalarF5E3 = 0_i32 / ScalarF5E3::ZERO; // 0/0 assert!(undefined.sign().is_undefined());
    /// ```
    pub fn sign(&self) -> Self {
        if self.is_undefined() {
            return *self;
        }
        if self.is_uniform() {
            return Self {
                fraction: SIGN_INDETERMINATE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.is_positive() {
            return Self::ONE;
        }
        Self::NEG_ONE
    }

    /// Returns the largest integer Scalar not greater than this value
    ///
    /// # Description
    ///
    /// The floor function maps a Scalar to the largest integer that is less than or equal to the value.
    ///
    /// # Returns
    ///
    /// - `[#.#]` ➔ `[#.0]` The largest integer not greater than the value
    /// - `[↑]` ➔ `[↑]` Exploded value unchanged
    /// - `[+↓]` ➔ `[0]` Zero
    /// - `[-↓]` ➔ `[-1]` Negative one
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rustI
    /// use spirix::{Scalar, ScalarF5E5};
    ///
    /// // The floor of the answer to everything is itself let integer = Scalar::<i32, i32>::from(42); assert!(integer.floor() == integer);
    ///
    /// // Floor of positive fractional number let fract_pos = ScalarF5E5::from(2.622057554); assert!(fract_pos.floor() == 2);
    ///
    /// // Floor of negative fractional number let fract_neg = ScalarF5E5::from(-24.1); assert!(fract_neg.floor() == -25);
    ///
    /// // Floor of Zero is Zero let zero = ScalarF5E5::ZERO; assert!(zero.floor() == 0);
    ///
    /// // Floor of positive vanished is Zero let tiny_pos = ScalarF5E5::MIN_POS / 12; assert!(tiny_pos.floor() == 0);
    ///
    /// // Floor of negative vanished is negative one let tiny_neg = ScalarF5E5::MIN_POS / -24; assert!(tiny_neg.floor() == -1);
    ///
    /// // Floor of exploded preserves the value let sploded = ScalarF5E5::MAX * ScalarF5E5::MAX; assert!(sploded.floor() == sploded);
    ///
    /// // Floor of Infinity preserves Infinity let infinity = ScalarF5E5::ONE / 0; assert!(infinity.floor() == infinity);
    ///
    /// // Floor of undefined remains undefined let undefined = ScalarF5E5::ZERO / 0; assert!(undefined.floor().is_undefined());
    /// ```
    pub fn floor(&self) -> Self {
        // Handle non-normal values first
        if !self.is_normal() {
            // Vanished values should floor to ZERO or NEG_ONE
            if self.vanished() {
                if self.is_negative() {
                    return Self::NEG_ONE;
                } else {
                    return Self::ZERO;
                }
            }
            // Other non-normal values (exploded, undefined, etc.) return themselves
            return *self;
        }

        // Recover the logical exponent (the actual power-of-2): stored ^ binade_origin. Value < 1 ⇔ logical k < 0.
        let logical_k_e: E = self.exponent ^ Self::binade_origin();
        let logical_k: isize = logical_k_e.saturate();
        if logical_k < 0 {
            if self.is_negative() {
                return Self::NEG_ONE;
            }
            return Self::ZERO;
        }
        let mut result = *self;
        // Already an integer when all FRAC bits are in the integer part: logical k ≥ FRAC - 1.
        if logical_k >= Self::fraction_bits().wrapping_sub(1) {
            return result;
        }
        // Fractional bit count = FRAC - logical_k - 1.
        let frac_bits = Self::fraction_bits().wrapping_sub(logical_k).wrapping_sub(1);
        let mask: F = !((F::one() << frac_bits).wrapping_sub(&F::one()));
        result.fraction = result.fraction & mask;
        result
    }

    /// Returns the smallest integer Scalar not less than this value
    ///
    /// # Description
    ///
    /// The ceiling function maps a Scalar to the smallest integer that is greater than or equal to the value.
    ///
    /// # Returns
    ///
    /// - `[#.#]` ➔ `[#.0]` The smallest integer not less than the value
    /// - `[↑]` ➔ `[↑]` Exploded value unchanged
    /// - `[+↓]` ➔ `[+1]` One
    /// - `[-↓]` ➔ `[0]` Zero
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF7E5};
    ///
    /// // The ceiling of an integer is itself let integer = Scalar::<i128, i32>::from(42_i128); assert!(integer.ceil() == integer);
    ///
    /// // Ceiling of positive fractional number let fract_pos = ScalarF7E5::E.pow(ScalarF7E5::PI); assert!(fract_pos.ceil() == 24_i128);
    ///
    /// // Ceiling of negative fractional number let fract_neg = ScalarF7E5::from(-24.5_f32); assert!(fract_neg.ceil() == -24_i128);
    ///
    /// // Ceiling of Zero is Zero let zero = ScalarF7E5::ZERO; assert!(zero.ceil() == 0_i128);
    ///
    /// // Ceiling of positive vanished is one let tiny_pos: ScalarF7E5 = ScalarF7E5::MIN_POS / 17_i128; assert!(tiny_pos.ceil() == 1_i128);
    ///
    /// // Ceiling of negative vanished is Zero let tiny_neg: ScalarF7E5 = ScalarF7E5::MAX_NEG / 29_i128; assert!(tiny_neg.ceil() == 0_i128);
    ///
    /// // Ceiling of exploded preserves the exploded state let sploded = ScalarF7E5::MAX * ScalarF7E5::MAX; assert!(sploded.ceil().exploded());
    ///
    /// // Ceiling of Infinity preserves Infinity let infinity: ScalarF7E5 = ScalarF7E5::ONE / 0_i128;  // Division by zero produces Infinity assert!(infinity.ceil().is_infinite());
    ///
    /// // Ceiling of undefined remains undefined let undefined: ScalarF7E5 = ScalarF7E5::ZERO / 0_i128;  // 0/0 is undefined assert!(undefined.ceil().is_undefined());
    /// ```
    pub fn ceil(&self) -> Self {
        let f = self.floor();
        // "Already integer" means floor equals self exactly, not just that the fraction bits coincide — NEG_ONE's stored fraction (0x00) matches any neg_one_normal-shaped self at any exponent.
        if f.fraction == self.fraction && f.exponent == self.exponent {
            f
        } else {
            f + Self::ONE
        }
    }

    /// Returns the nearest integer Scalar
    ///
    /// # Description
    ///
    /// Rounds this Scalar to the nearest integer value using banker's rounding. Ties (exactly integer + 1/2) are rounded to the nearest even integer.
    ///
    /// # Returns
    ///
    /// - `[#.#]` ➔ `[#.0]` The nearest integer
    /// - `[↑]` ➔ `[↑]` Exploded value unchanged
    /// - `[↓]` ➔ `[0]` Zero
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF7E6};
    ///
    /// // Rounding an integer returns the same integer let integer = Scalar::<i128, i64>::from(42_i128); assert!(integer.round() == integer);
    ///
    /// // Rounding positive numbers let tau = ScalarF7E6::TAU; assert!(tau.round() == 6_i128);
    ///
    /// // Ties use banker's rounding (round half to even): 2.5 → 2, 1.5 → 2 let two_and_half: ScalarF7E6 = ScalarF7E6::from(2_i128) + ScalarF7E6::ONE / 2_i128; assert!(two_and_half.round() == 2_i128); let one_and_half: ScalarF7E6 = ScalarF7E6::ONE + ScalarF7E6::ONE / 2_i128; assert!(one_and_half.round() == 2_i128);
    ///
    /// // Rounding negative numbers let series: ScalarF7E6 = ScalarF7E6::NEG_ONE / 12_i128; assert!(series.round() == 0_i128);
    ///
    /// // Rounding Zero returns Zero let zero = ScalarF7E6::ZERO; assert!(zero.round() == 0_i128);
    ///
    /// // Vanished values round to Zero let tiny: ScalarF7E6 = ScalarF7E6::MIN_POS / 17_i128; assert!(tiny.round() == 0_i128);
    ///
    /// // Exploded values remain unchanged let huge = ScalarF7E6::MAX * ScalarF7E6::MAX; assert!(huge.round().fraction == huge.fraction && huge.round().exponent == huge.exponent);
    ///
    /// // Infinity remains unchanged let infinity: ScalarF7E6 = ScalarF7E6::ONE / 0_i128; assert!(infinity.round().fraction == infinity.fraction && infinity.round().exponent == infinity.exponent);
    ///
    /// // Undefined Scalars maintain their undefined state let undefined: ScalarF7E6 = ScalarF7E6::ZERO / 0_i128; assert!(undefined.round().fraction == undefined.fraction && undefined.round().exponent == undefined.exponent);
    /// ```
    pub fn round(&self) -> Self {
        if !self.is_normal() {
            if self.vanished() {
                return Self::ZERO;
            }
            return *self;
        }
        // Recover logical k: stored ^ binade_origin.
        let logical_k_e: E = self.exponent ^ Self::binade_origin();
        let e: isize = logical_k_e.saturate();
        // logical k < -1 means |value| < 0.5 — round to 0. At k=-1 (|v| in [0.5, 1]) the tie is value = ±0.5 / ±1; banker's still rounds 0.5 → 0 (even).
        if e < -1 {
            return Self::ZERO;
        }
        let f = self.floor();
        // Already integer: check both fraction and exponent match.
        if f.fraction == self.fraction && f.exponent == self.exponent {
            return f;
        }
        // Already integer when logical k >= FRAC - 1.
        if e >= Self::fraction_bits().wrapping_sub(1) {
            return f;
        }
        // Guard bit (0.5 position) at FRAC - logical_k - 2.
        let guard_pos = Self::fraction_bits().wrapping_sub(e).wrapping_sub(2);
        let guard = (self.fraction >> guard_pos) & F::one();
        if guard == F::zero() {
            return f;
        } // < 0.5, floor
        let sticky_mask: F = (F::one() << guard_pos).wrapping_sub(&F::one());
        let sticky = self.fraction & sticky_mask;
        if sticky != F::zero() {
            return f + Self::ONE;
        } // > 0.5, ceil
          // Exactly 0.5: banker's — round to even. int_lsb at position guard_pos + 1 = FRAC - e - 1, always a real stored bit (no implicit-sign-bit special case needed since logical e=0 spans [1,2) and has FRAC-1 as an actual integer ones bit).
        let int_lsb = (self.fraction >> guard_pos.wrapping_add(1)) & F::one();
        if int_lsb != F::zero() {
            f + Self::ONE
        } else {
            f
        } // odd rounds up, even stays
    }

    /// Returns the fractional part of this Scalar
    ///
    /// # Description
    ///
    /// Extracts the fractional part of this Scalar by subtracting its floor, following the mathematical definition: frac(x) = x - ⌊x⌋ This returns a value in the range [0,1). Future implementations will utilize bit masking and normalization
    ///
    /// # Returns
    ///
    /// - `[#.#]` ➔ `[+0.#]` The fractional part (value - floor(value))
    /// - `[#.0]` ➔ `[0]` Zero for integers
    /// - `[↑]` ➔ `[0]` Zero for exploded values
    /// - `[∞]` ➔ `[℘?]` Undefined state
    /// - `[+↓]` ➔ `[+↓]` The positive vanished value itself
    /// - `[-↓]` ➔ `[1-ε]` Value approaching 1
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF7E4};
    ///
    /// // Fractional part of an integer is Zero let integer = Scalar::<i128, i16>::from(42); assert!(integer.frac() == 0);
    ///
    /// // Fractional part of positive number let pi = ScalarF7E4::PI; assert!(pi - pi.frac() == pi.floor()); assert!(pi.frac() > 0 && pi.frac() < 1); // ~0.14159...
    ///
    /// // Fractional part of negative number let neg = ScalarF7E4::from(-3.25); assert!(neg.frac() == 0.75);
    ///
    /// // Fractional part of Zero is Zero let zero = ScalarF7E4::ZERO; assert!(zero.frac() == 0);
    ///
    /// // Fractional part of positive vanished returns the value itself let tiny_pos: ScalarF7E4 = ScalarF7E4::MIN_POS / 12_i128; assert!(tiny_pos.frac().vanished() && tiny_pos.frac().is_positive());
    ///
    /// // Fractional part of negative vanished approaches 1 let tiny_neg: ScalarF7E4 = ScalarF7E4::MIN_POS / -67_i128; assert!(tiny_neg.frac() - ScalarF7E4::EFFECTIVELY_POS_ONE == 0_i128);
    ///
    /// // Fractional part of exploded values is Zero let huge = ScalarF7E4::MAX.square(); assert!(huge.frac() == ScalarF7E4::ZERO);
    ///
    /// // Fractional part of Infinity is undefined let infinity: ScalarF7E4 = ScalarF7E4::ONE / 0_i128; assert!(infinity.frac().is_undefined());
    ///
    /// // Fractional part of undefined remains undefined let undefined: ScalarF7E4 = ScalarF7E4::ZERO / 0_i128; assert!(undefined.frac().is_undefined());
    /// ```
    pub fn frac(&self) -> Self {
        if !self.is_normal() {
            if self.vanished() && self.is_negative() {
                return Self::EFFECTIVELY_POS_ONE;
            }
            if self.exploded() {
                return Self::ZERO;
            }
            if self.is_infinite() {
                return Self {
                    fraction: FRACTIONAL_INFINITY.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }
        *self - self.floor()
    }
    /// Returns the larger of this Scalar and another
    ///
    /// # Description
    ///
    /// Compares two Scalar values and returns the greater one. This provides the mathematical maximum function for two values.
    ///
    /// # Arguments
    ///
    /// * `other` - The Scalar to compare with this one
    ///
    /// # Returns
    ///
    /// - `[℘?]` & `[?]` ➔ `[℘?]` The first undefined state encountered
    /// - `[∞]` & `[?]` ➔ `[℘ ⌈]` Undefined max state
    /// - `[#]` & `[#]` ➔ `[#]` The larger of `self` and `other` based on mathematical ordering
    /// - `[+]` & `[-]` ➔ `[+]` The positive value
    /// - `[0]` & `[+]` ➔ `[+]` The positive value
    /// - `[0]` & `[-]` ➔ `[0]` Zero
    /// - `[+↑]` & `[-↑]` ➔ `[+↑]` The positive exploded
    /// - `[+↑]` & `[+↑]` ➔ `[℘ ⌈]` Undefined max state
    /// - `[+↓]` & `[-↓]` ➔ `[+↓]` The positive vanished
    /// - `[+↓]` & `[+↓]` ➔ `[℘ ⌈]` Undefined max state
    /// - `[?]` & `[?]` ➔ `[#]` The larger of `self` and `other` based on mathematical ordering
    ///
    /// # Ordering
    /// - `[+↑]` Positive exploded (greatest)
    /// - `[+#]` Positive normal
    /// - `[+↓]` Positive vanished
    /// - `[0]` Zero
    /// - `[-↓]` Negative vanished
    /// - `[-#]` Negative normal
    /// - `[-↑]` Negative exploded (least) -
    /// - `[℘?]` or `[∞]` Unordered!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E6};
    ///
    /// // Normal values let a = Scalar::<i64, i64>::from(42_i64); let b = ScalarF6E6::from(21_i64); assert!(a.max(b) == a);
    ///
    /// // With negative values let neg = ScalarF6E6::from(-12_i64); assert!(neg.max(b) == b);
    ///
    /// // With Zero let zero = ScalarF6E6::ZERO; assert!(neg.max(zero) == zero);
    ///
    /// // With vanished values let tiny_pos: ScalarF6E6 = ScalarF6E6::MIN_POS / 4_i64; assert!(tiny_pos.max(neg).is_positive());
    ///
    /// // With exploded values let huge = ScalarF6E6::MIN * ScalarF6E6::MAX; assert!(a.max(5_i64) == a);
    ///
    /// // With Infinity (mathematically undefined comparison) let infinity: ScalarF6E6 = 1_i64 / zero; assert!(a.max(infinity).is_undefined()); assert!(infinity.max(a).is_undefined());
    ///
    /// // Vanished values with same sign are not comparable let little = ScalarF6E6::MIN_POS.square(); assert!(tiny_pos.max(little).is_undefined());
    ///
    /// // With undefined let undefined: ScalarF6E6 = infinity * ScalarF6E6::ZERO; assert!(a.max(undefined).is_undefined()); assert!(undefined.max(a).is_undefined());
    /// ```
    pub fn max<S>(&self, other: S) -> Self
    where
        S: Into<Self>,
    {
        let other = other.into();

        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return other;
            }

            if self.is_infinite() || other.is_infinite() {
                return Self {
                    fraction: MAX_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            if self.vanished()
                && other.vanished()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MAX_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            if self.exploded()
                && other.exploded()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MAX_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
        if *self > other {
            *self
        } else {
            other
        }
    }

    /// Returns the smaller of this Scalar and another
    ///
    /// # Description
    ///
    /// Compares two Scalar values and returns the lesser one. This provides the mathematical minimum function for two values.
    ///
    /// # Arguments
    ///
    /// * `other` - The Scalar to compare with this one
    ///
    /// # Returns
    ///
    /// - `[℘?]` & `[?]` ➔ `[℘?]` The first undefined state encountered
    /// - `[∞]` & `[?]` ➔ `[℘ ⌊]` undefined min state
    /// - `[#]` & `[#]` ➔ `[#]` The smaller of `self` and `other` based on mathematical ordering
    /// - `[-]` & `[+]` ➔ `[-]` The negative value
    /// - `[0]` & `[+]` ➔ `[0]` Zero
    /// - `[0]` & `[-]` ➔ `[0]` The negative value
    /// - `[-↑]` & `[+↑]` ➔ `[-↑]` The negative exploded
    /// - `[+↑]` & `[+↑]` ➔ `[℘ ⌊]` undefined min state
    /// - `[-↓]` & `[+↓]` ➔ `[-↓]` The negative vanished
    /// - `[+↓]` & `[+↓]` ➔ `[℘ ⌊]` undefined min state
    /// - `[?]` & `[?]` ➔ `[#]` The smaller of `self` and `other` based on mathematical ordering
    ///
    /// # Ordering
    /// - `[+↑]` Positive exploded (greatest)
    /// - `[+#]` Positive normal
    /// - `[+↓]` Positive vanished
    /// - `[0]` Zero
    /// - `[-↓]` Negative vanished
    /// - `[-#]` Negative normal
    /// - `[-↑]` Negative exploded (least) -
    /// - `[℘?]` or `[∞]` Unordered!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E6};
    ///
    /// // Normal values let a = Scalar::<i32, i64>::from(42); // Using turbofish notation let b = ScalarF5E6::from(21); // Using shorthand assert!(a.min(b) == b); // Using owned value assert!(a.min(&b) == b); // Using reference
    ///
    /// // With negative values let neg = ScalarF5E6::from(-42); assert!(a.min(neg) == neg);
    ///
    /// // With Zero assert!(neg.min(0) == neg);
    ///
    /// // With Infinity (mathematically undefined comparison) let infinity: ScalarF5E6 = ScalarF5E6::ONE / 0_i32; assert!(a.min(infinity).is_undefined()); assert!(infinity.min(a).is_undefined());
    ///
    /// // With vanished values let vanished_neg: ScalarF5E6 = ScalarF5E6::MIN_POS / -16_i32; assert!(vanished_neg.min(a).is_negative());
    ///
    /// // With exploded values let exploded: ScalarF5E6 = ScalarF5E6::MAX / 0.0625_f32; assert!(a.min(exploded) == a);
    ///
    /// // Vanished values with same sign are not comparable let small: ScalarF5E6 = ScalarF5E6::MAX_NEG / 512_i32; assert!(vanished_neg.min(small).is_undefined());
    ///
    /// // With undefined let undefined: ScalarF5E6 = ScalarF5E6::ZERO / 0_i32;  // 0/0 is undefined assert!(a.min(undefined).is_undefined()); assert!(undefined.min(a).is_undefined());
    /// ```
    pub fn min<S>(&self, other: S) -> Self
    where
        S: Into<Self>,
    {
        let other = other.into();

        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return other;
            }

            if self.is_infinite() || other.is_infinite() {
                return Self {
                    fraction: MIN_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            if self.vanished()
                && other.vanished()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MIN_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            if self.exploded()
                && other.exploded()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MIN_UNORDERED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
        if *self < other {
            *self
        } else {
            other
        }
    }
    /// Constrains a Scalar value between a minimum and maximum value
    ///
    /// # Description
    ///
    /// Clamps this Scalar's value to be within a specified range. If this value is less than the minimum, returns the minimum. If greater than the maximum, returns the maximum. Otherwise, returns this value unchanged.
    ///
    /// # Arguments
    ///
    /// * `min` - The lower bound for clamping
    /// * `max` - The upper bound for clamping
    ///
    /// # Returns
    ///
    /// - `[℘?]` & `[?]` & `[?]` ➔ `[℘?]` The first undefined state encountered
    /// - `[∞]` & `[?]` & `[?]` ➔ `[℘ ∩]` Undefined clamp state
    /// - `[?]` & `[∞]` & `[?]` ➔ `[℘ ∩]` Undefined clamp state
    /// - `[?]` & `[?]` & `[∞]` ➔ `[℘ ∩]` Undefined clamp state
    /// - `[?]` & `[>]` & `[<]` ➔ `[℘ ∩]` Undefined clamp state (min > max)
    /// - `[<min]` & `[min]` & `[max]` ➔ `[min]` Lower bound
    /// - `[>max]` & `[min]` & `[max]` ➔ `[max]` Upper bound
    /// - `[val]` & `[min]` & `[max]` ➔ `[val]` Value unchanged if within range
    /// - `[↓]` & `[↓]` & `[?]` ➔ `[℘ ∩]` Undefined clamp state (same sign vanished bounds)
    /// - `[↑]` & `[↑]` & `[?]` ➔ `[℘ ∩]` Undefined clamp state (same sign exploded bounds)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF4E3};
    ///
    /// // Normal value clamping with Scalar bounds let value = ScalarF4E3::from(42_i16); let min = ScalarF4E3::from(8_i16); let max = ScalarF4E3::from(32_i16); assert!(value.clamp(min, max) == max);
    ///
    /// // With mixed primitive types let value2 = ScalarF4E3::from(42_i16); assert!(value2.clamp(8i16, 32f32) == ScalarF4E3::from(32_i16)); assert!(value2.clamp(20u8, 100i64) == value2); // within range
    ///
    /// // Value within range let in_range = ScalarF4E3::from(16_i16); assert!(in_range.clamp(min, max) == in_range);
    ///
    /// // Value below range let below = ScalarF4E3::from(4_i16); assert!(below.clamp(min, max) == min);
    ///
    /// // With Infinity (mathematically undefined) let infinity: ScalarF4E3 = ScalarF4E3::ONE / 0_i16; assert!(value.clamp(min, infinity).is_undefined()); assert!(value.clamp(infinity, max).is_undefined()); assert!(infinity.clamp(min, max).is_undefined());
    ///
    /// // With special values let zero = ScalarF4E3::ZERO; let tiny_pos: ScalarF4E3 = ScalarF4E3::MIN_POS / 64_i16; let neg = ScalarF4E3::from(-16_i16);
    ///
    /// // Clamping between negative and positive assert!(neg.clamp(neg, zero) == neg); assert!(tiny_pos.clamp(neg, zero) == zero);
    ///
    /// // With undefined let undefined: ScalarF4E3 = ScalarF4E3::ZERO / 0_i16; // 0/0 undefined state assert!(value.clamp(min, &undefined).is_undefined()); assert!(undefined.clamp(&min, max).is_undefined());
    ///
    /// // Unordered bounds (min > max) produce undefined assert!(value.clamp(max, min).is_undefined());
    /// ```
    pub fn clamp<L, R>(&self, min: L, max: R) -> Self
    where
        L: Into<Self>,
        R: Into<Self>,
    {
        let min = min.into();
        let max = max.into();

        if self.is_undefined() {
            return *self;
        }
        if min.is_undefined() {
            return min;
        }
        if max.is_undefined() {
            return max;
        }
        if self.is_infinite() || min.is_infinite() || max.is_infinite() {
            return Self {
                fraction: CLAMP_UNORDERED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if min > max {
            return Self {
                fraction: CLAMP_UNORDERED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        let clamped = self.max(min).min(max);
        if clamped.is_undefined() {
            return Self {
                fraction: CLAMP_UNORDERED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        clamped
    }

    /// Determines if this Scalar represents a prime number
    #[cfg(feature = "prime")]
    ///
    /// # Description
    ///
    /// Prime numbers are natural integer numbers greater than 1 that are only divisible by 1 and themselves. This method provides deterministic primality testing for numbers up to 2^64, and probabilistic testing with extremely high confidence levels for larger numbers.
    ///
    /// # Returns
    ///
    /// - `['ℕ]` ➔ `true` Prime numbers
    /// - `[!'ℕ]` ➔ `false` Composite numbers
    /// - `[0]` ➔ `false` Zero
    /// - `[1]` ➔ `false` One
    /// - `[2]` ➔ `true` Two
    /// - `[#.#]` ➔ `false` Non-integers
    /// - `[-#]` ➔ `false` negative numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF7E3};
    ///
    /// assert!(!Scalar::<i128,i8>::ZERO.is_prime());   // 0 is not prime assert!(!ScalarF7E3::ONE.is_prime());           // 1 is not prime assert!(ScalarF7E3::TWO.is_prime());            // 2 is prime assert!(ScalarF7E3::from(3).is_prime());        // 3 is prime assert!(!ScalarF7E3::from(4).is_prime());       // 4 is not prime assert!(ScalarF7E3::from(17).is_prime());       // 17 is prime assert!(!ScalarF7E3::from(68).is_prime());      // 68 is not prime
    ///
    /// // Negative numbers are not prime assert!(!ScalarF7E3::NEG_ONE.is_prime()); assert!(!ScalarF7E3::from(-7).is_prime());
    ///
    /// // Non-integer values are not prime assert!(!ScalarF7E3::PI.is_prime()); assert!(!ScalarF7E3::from(2.5).is_prime());
    /// ```
    #[inline]
    pub fn is_prime(&self) -> bool {
        // Range: value must be integer ≥ 2 and ≤ 2^FRAC. Logical k in [1, FRAC).
        let logical_k_e: E = self.exponent ^ Self::binade_origin();
        let logical_k: isize = logical_k_e.saturate();
        if logical_k < 1 || logical_k >= Self::fraction_bits() {
            return false;
        }

        if self.is_negative() {
            return false;
        }

        if *self == 2 {
            return true;
        }

        if self % 2 == 0 {
            return false;
        }

        let modulo = self % 2;
        if modulo != 1 {
            return false;
        }

        if self > u64::MAX {
            let integer: u128 = self.to_u128();
            match num_prime::nt_funcs::is_prime(
                &integer,
                Some(num_prime::PrimalityTestConfig::default()),
            ) {
                num_prime::Primality::Yes => true,
                num_prime::Primality::Probable(_) => true, // Accept probable primes as prime
                num_prime::Primality::No => false,
            }
        } else {
            let integer: u64 = (*self).to_u64();
            num_prime::nt_funcs::is_prime64(integer)
        }
    }

    /// Normalizes this Scalar by restoring the sign, shifting out leading redundancy, then stripping the sign back off. Adjusts the exponent accordingly. If shifting would cause the exponent to underflow, the value becomes vanished.
    #[inline]
    pub fn normalize(&mut self) {
        // Restore the sign to count leading same bits in the full value
        let shift = self
            .fraction
            .leading_ones()
            .max(self.fraction.leading_zeros());
        if shift > 0 {
            let shift = shift as isize;
            if shift == Self::fraction_bits() {
                // All bits identical — either zero (all 0s) or all 1s
                if !self.is_negative() {
                    // All zeros in stored = most negative effective, but with max leading same bits — effectively zero.
                    self.exponent = Self::ambiguous_exponent();
                    return;
                }
                // All ones in stored — uniform pattern, falls thru to the cycle-widened path below.
            }

            // AMBIG=0 underflow detection via cycle math: widen, subtract, bounds-check. New cycle position < min_pos (= 1) means we wrapped past AMBIG → vanished.
            let pa = self.exponent.cycle_widen();
            let shift_e: E = shift.as_();
            let w_shift = shift_e.sign_extend();
            let new_pos = pa.w_sub(w_shift);
            let min_pos = Self::min_exponent().cycle_widen();
            if new_pos < min_pos {
                self.exponent = Self::ambiguous_exponent();
                self.fraction = self.fraction << (shift.wrapping_sub(1));
            } else {
                self.exponent = new_pos.deflate();
                self.fraction = self.fraction << shift;
            }
        }
    }
    #[inline]
    /// Normalizes an exploded Scalar by shifting the fraction left until the most significant bit is in the N-1 position, exponent is not touched.
    ///
    /// Example bit positions: 01234567... □■xxxxxx... - Exploded positive numbers ■□xxxxxx... - Exploded negative numbers
    pub(crate) fn normalize_exploded(&mut self) {
        let shift = self
            .fraction
            .leading_ones()
            .max(self.fraction.leading_zeros());
        self.fraction = self.fraction << (shift.wrapping_sub(1) as usize);
    }

    #[inline]
    /// Normalizes a vanished Scalar by shifting its fraction to the N-2 position. Sign bits occupy N-0 and N-1, exponent is not touched
    ///
    /// Example bit positions: 01234567... □□■xxxxx... - Vanished positive numbers ■■□xxxxx... - Vanished negative numbers
    pub(crate) fn normalize_vanished(&mut self) {
        let shift = self
            .fraction
            .leading_ones()
            .max(self.fraction.leading_zeros());
        if shift > 2 {
            self.fraction = self.fraction << (shift.wrapping_sub(2) as usize);
        } else if shift < 2 {
            self.fraction = self.fraction >> (2.wrapping_sub(&shift) as usize);
        }
    }
}

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
#[allow(dead_code)]
fn _printey<T: core::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = core::mem::size_of::<T>().wrapping_shl(3);
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits.wrapping_sub(1) && b % 8 == 7 {
            result.push(' ');
        }
        if b == bits / 2 - 1 {
            result.push(' '); // Extra space at center
        }
    }
    result
}
