use crate::constants::ScalarConstants;
use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar};
use i256::I256;
use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use core::ops::*;

macro_rules! impl_scalar_new {
    ($($f:ty, $e:ty);*) => {
        $(
impl Scalar<$f, $e> {
    /// Creates a new Scalar from raw fraction and exponent integers.
    ///
    /// This is a low-level constructor that directly sets the internal state.
    /// For normal number creation, use `from()` which handles:
    /// - Proper normalization of the fraction
    /// - Exponent calculation
    /// - Special value mapping
    ///
    /// # Example
    /// ```
    /// # use spirix::Scalar;
    /// let raw_scalar = Scalar::<i32, i8>::new(0b10101 << 26, 6);
    /// let normal = Scalar::<i32, i8>::from(42);
    /// assert!(raw_scalar == normal);
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
    /// This forms a behavioral prefix that indicates the value's classification:  
    ///  
    /// N0: Zero and Infinity  
    /// □□□□□□□□  Zero `[0]`  
    /// ■■■■■■■■  Infinity `[∞]`  
    ///  
    /// N1: Normal and Exploded Values  
    /// □■xxxxxx  Positive `[+#]`  
    /// ■□xxxxxx  Negative `[-#]`  
    ///  
    /// N2: Vanishing Values
    /// □□■xxxxx  Positive effectively zero `[+↓]`  
    /// ■■□xxxxx  Negative effectively zero `[-↓]`  
    ///  
    /// N3 - N-7: Undefined  
    /// □□□■xxxx | ■■■□xxxx `[℘?]`  
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
    /// Normal numbers have a definite magnitude and participate fully in all arithmetic operations.
    /// Unlike Infinity, Zero, escaped, or undefined values, normal numbers occupy the "standard" region of numeric space where arithmetic behaves conventionally.
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
    /// // Regular values are normal
    /// let normal = Scalar::<i16, i16>::from(42);
    /// assert!(normal.is_normal());
    ///
    /// // Normal values maintain their normality thru standard operations
    /// let still_normal = normal * ScalarF4E4::PI / 2;
    /// assert!(still_normal.is_normal());
    ///
    /// // Zero is not normal
    /// let zero = ScalarF4E4::ZERO;
    /// assert!(!zero.is_normal());
    ///
    /// // Infinity is not normal
    /// let infinity = ScalarF4E4::ONE / 0;
    /// assert!(!infinity.is_normal());
    ///
    /// // Exploded values are not normal
    /// let exploded = ScalarF4E4::MAX + ScalarF4E4::MAX;
    /// assert!(!exploded.is_normal());
    ///
    /// // Vanished values are not normal
    /// let vanished = ScalarF4E4::MIN_POS - ScalarF4E4::MIN_POS * 1.25;
    /// assert!(!vanished.is_normal());
    ///
    /// // Undefined Scalars are definitely not normal
    /// let undefined = zero / 0;
    /// assert!(!undefined.is_normal());
    ///
    /// // Operations that exceed representable range escape normality
    /// let no_longer_normal = ScalarF4E4::MAX_NEG.square();
    /// assert!(!no_longer_normal.is_normal());
    /// ```
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.exponent != E::AMBIGUOUS_EXPONENT
    }

    /// Checks if this Scalar is undefined `[℘?]`
    ///
    /// # Description
    ///
    /// Undefined Scalars represent operations that have no known or agreed upon mathematical result.
    /// Spirix uses specific bit patterns to track various types of undefined states, maintaining "first cause" information.
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
    /// // Zero over Zero creates an undefined state
    /// let undefined = Scalar::<i32, i8>::ZERO / 0;
    /// assert!(undefined.is_undefined());
    ///
    /// // Normal numbers are defined
    /// let normal = ScalarF5E3::from(42);
    /// assert!(!normal.is_undefined());
    ///
    /// // Zero is defined
    /// let zero = ScalarF5E3::ZERO;
    /// assert!(!zero.is_undefined());
    ///
    /// // Escaped values are defined
    /// let exploded = ScalarF5E3::MAX * ScalarF5E3::MIN;
    /// assert!(!exploded.is_undefined());
    ///
    /// // Exploded values in certain operations stay defined
    /// let vanished = 1 / exploded;
    /// assert!(!vanished.is_undefined());
    ///
    /// // But some operations produce undefined results
    /// let undefined_exploded_add = exploded + 1;
    /// assert!(undefined_exploded_add.is_undefined());
    ///
    /// // Wheras others do not
    /// let one = vanished + 1;
    /// assert!(one == 1);
    ///
    /// // Infinity is defined
    /// let infinity = normal / 0;
    /// assert!(!infinity.is_undefined());
    /// ```
    #[inline]
    pub fn is_undefined(&self) -> bool {
        // Zero and Infinity are defined
        if self.is_n0() {
            return false;
        }

        let prefix = self.prefix();
        // Check if top 3 bits are equal by pushing 5 bits off
        // ↓↓↓                ↓↓↓
        // □□□xxxxx -5-> □□□□□□□□ - Undefined (℘)
        let top_three = prefix >> 5;
        // Then rotate and compare.  If uniform, they will be the same
        top_three == top_three.rotate_right(1)
    }

    pub fn is_n0(&self) -> bool {
        // Check for uniform patterns by shifting and equality
        self.prefix() == self.prefix().rotate_right(1)
    }

    pub fn is_n1(&self) -> bool {
        let prefix = self.prefix();
        // Check for N1 patterns by shifting:
        // □■xxxxxx -6-> □□□□□□□■
        // ■□xxxxxx -6-> ■■■■■■■□
        let top_two = prefix >> 6;
        top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8
    }

    pub fn is_n2(&self) -> bool {
        let prefix = self.prefix();
        // Check for N2 patterns by shifting:
        // □□■xxxxx -5-> □□□□□□□■
        // ■■□xxxxx -5-> ■■■■■■■□
        let top_three = prefix >> 5;
        top_three == 0b00000001u8 as i8 || top_three == 0b11111110u8 as i8
    }

    /// Checks if this Scalar's magnitude is negligible `[0]`, `[↓]`
    ///
    /// # Description
    ///
    /// Negligible values have effectively zero magnitude.
    /// This includes both actual Zero `[0]` and vanished values `[↓]` that have become so small they no longer meaningfully contribute to addition or subtraction.
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
    /// // Zero: The original negligible number
    /// let zero = Scalar::<i16, i32>::ZERO;
    /// assert!(zero.is_negligible());
    ///
    /// // A number so small it's effectively zero
    /// let vanished = ScalarF4E5::MIN_POS / 57;
    /// assert!(vanished.is_negligible());
    ///
    /// // Even tiny numbers maintain their sign
    /// let neg_vanished = vanished * -1;
    /// assert!(neg_vanished.is_negative());
    ///
    /// // Normal values are not negligible
    /// let meaning = ScalarF4E5::from(1729);
    /// assert!(!meaning.is_negligible());
    ///
    /// // Large escaped values are not negligible
    /// let exploded = ScalarF4E5::MAX * ScalarF4E5::MIN;
    /// assert!(!exploded.is_negligible());
    ///
    /// // Undefined Scalars are not negligible
    /// let undefined = ScalarF4E5::ZERO / 0;
    /// assert!(!undefined.is_negligible());
    ///
    /// // Infinite Scalars are not negligible
    /// let infinite = ScalarF4E5::ONE / 0;
    /// assert!(!infinite.is_negligible());
    ///
    /// // Reciprocals of Infinite Scalars are Zero!
    /// let infinite = 1 / infinite;
    /// assert!(infinite.is_negligible());
    /// assert!(infinite == 0);
    ///
    /// // Division by a vanished value creates an exploded result
    /// let exploded = 1 / tiny;
    /// assert!(!exploded.is_negligible());
    /// ```
    #[inline]
    pub fn is_negligible(&self) -> bool {
        // Extract high byte and cast to signed to use arithmetic shifts
        let prefix = self.prefix();
        if prefix == 0 {
            return true;
        }

        self.is_n2()
    }

    /// Returns true if this Scalar is an infinitesimal value `[↓]`
    /// (close but not equal to Zero)
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
    /// // Create a ridiculously small positive number
    /// let vanished = Scalar::<i16, i8>::MIN_POS / 123.45;
    /// assert!(vanished.vanished());
    /// assert!(vanished.is_positive());
    ///
    /// // And a ridiculously small negative number
    /// let vanished_negative = ScalarF4E3::MIN_POS / -12;
    /// assert!(vanished_negative.vanished());
    /// assert!(vanished_negative.is_negative());
    ///
    /// // Actual Zero is not vanished - it's truly Zero
    /// let actual_zero = ScalarF4E3::ZERO;
    /// assert!(!actual_zero.vanished());
    ///
    /// // Normal values are not vanished
    /// let normal = ScalarF4E3::from(42);
    /// assert!(!normal.vanished());
    ///
    /// // Large escaped values aren't vanished - they're exploded!
    /// let ginormous = ScalarF4E3::MAX.pow(ScalarF4E3::MAX);
    /// assert!(!ginormous.vanished());
    /// assert!(ginormous.exploded());
    ///
    /// // Normal division by a vanished value produces an exploded result
    /// let exploded = 1 / vanished;
    /// assert!(exploded.exploded());
    /// assert!(!exploded.vanished());
    ///
    /// // Reciprocal of Infinity is not vanished, it's Zero!
    /// let infinity = ScalarF4E3::from(42) / 0;
    /// let zero = 1 / infinity;
    /// assert!(!zero.vanished());
    /// assert!(zero.is_zero());
    /// ```
    #[inline]
    pub fn vanished(&self) -> bool {
        self.is_n2()
    }

    /// Returns true if this Scalar is ridiculously large `[↑]` but not ∞
    ///
    /// # Description
    ///
    /// Exploded values are numbers that have grown so large their magnitude can no longer be recorded, but they maintain their sign information.
    /// They participate meaningfully in absolute operations like multiplication and division operations, but relative operations like addition and subtraction with exploded or infinite Scalars will produce undefined results.
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
    /// use spirix::{Scalar, ScalarF7E7};
    ///
    /// // Create an astronomically large positive Scalar
    /// let biggin = Scalar::<i128, i128>::MAX.pow(ScalarF7E7::MAX);
    /// assert!(biggin.exploded());
    /// assert!(biggin.is_positive());
    /// // Scalars don't saturate to infinity
    /// assert!(!biggin.is_infinite());
    ///
    /// // Create an exploded negative Scalar
    /// let huge_negative = ScalarF7E7::MIN.pow(ScalarF7E7::PI);
    /// assert!(huge_negative.exploded());
    /// assert!(huge_negative.is_negative());
    ///
    /// // Actual Zero is not vanished
    /// let actual_zero = ScalarF7E7::ZERO;
    /// assert!(!actual_zero.vanished());
    ///
    /// // Infinity is not exploded either
    /// let infinity = 1 / actual_zero;
    /// assert!(!infinity.exploded());
    ///
    /// // Normal values are, well, normal!
    /// let normal = ScalarF7E7::from(42);
    /// assert!(normal.is_normal());
    ///
    /// // Small escaped values aren't Zero - they're vanished!
    /// let vanished = ScalarF7E7::MIN_POS.square();
    /// assert!(!vanished.is_zero());
    /// assert!(vanished.vanished());
    ///
    /// // Multiplying or dividing exploded Scalars maintains the exploded state
    /// let still_exploded = huge / 42;
    /// assert!(still_exploded.exploded());
    /// let definitely_exploded = huge * 42;
    /// assert!(definitely_exploded.exploded());
    ///
    /// // Division by vanished values produces exploded results
    /// let also_exploded = 1 / tiny;
    /// assert!(also_exploded.exploded());
    /// ```
    #[inline]
    pub fn exploded(&self) -> bool {
        self.exponent == E::AMBIGUOUS_EXPONENT && self.is_n1()
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
    /// // Exploded Scalars are transfinite
    /// let exploded = ScalarF5E4::MAX * ScalarF5E4::MAX;
    /// assert!(exploded.is_transfinite());
    /// assert!(exploded.exploded());
    /// assert!(!exploded.is_infinite()); // But it's not infinity
    ///
    /// // Exploded negatives are also transfinite
    /// let neg_exploded = exploded * -1;
    /// assert!(neg_exploded.is_transfinite());
    /// assert!(neg_exploded.is_negative());
    ///
    /// // Division by zero produces true mathematical infinity
    /// let infinity = ScalarF5E4::ONE / 0;
    /// assert!(infinity.is_transfinite());
    /// assert!(!infinity.exploded()); // Not the same as exploded!
    ///
    /// // Normal values are not transfinite
    /// let normal = ScalarF5E4::from(42);
    /// assert!(!normal.is_transfinite());
    ///
    /// // Zero is not transfinite
    /// let zero = ScalarF5E4::ZERO;
    /// assert!(!zero.is_transfinite());
    ///
    /// // Vanished values are not transfinite (they're the opposite!)
    /// let tiny = ScalarF5E4::MAX_NEG.square();
    /// assert!(!tiny.is_transfinite());
    ///
    /// // The reciprocal of transfinite is tiny
    /// let vanished = 1 / exploded;
    /// let also_zero = 1 / infinity;
    /// assert!(vanished.is_negligible());
    /// assert!(also_zero.is_negligible());
    ///
    /// // Anything over infinity is exactly Zero
    /// let zero_again = exploded / infinity;
    /// assert!(zero_again.is_zero());
    /// // Except for infinity over infinity, of course
    /// let undefined = ScalarF5E4::INFINITY / infinity;
    /// assert!(undefined.is_undefined());
    ///
    /// // Math operations with infinity follow mathematical rules
    /// let also_infinity = infinity * ScalarF5E4::PI;
    /// assert!(also_infinity.is_transfinite());
    /// let still_infinity = infinity.pow(-ScalarF5E4::E);
    /// assert!(still_infinity.is_transfinite());
    /// ```
    pub fn is_transfinite(&self) -> bool {
        self.exponent == E::AMBIGUOUS_EXPONENT && (self.is_n1() || self.fraction == F::NEG_ONE)
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
    /// // Normal values are finite
    /// let normal = Scalar::<i32, i16>::from(42);
    /// assert!(normal.is_finite());
    ///
    /// // Zero is finite
    /// let zero = normal - normal;
    /// assert!(zero.is_finite());
    ///
    /// // Pi is finite
    /// let pi = ScalarF5E4::PI;
    /// assert!(pi.is_finite());
    ///
    /// // Exploded values are not finite
    /// let exploded = ScalarF5E4::MAX * 2 ;
    /// assert!(!exploded.is_finite());
    ///
    /// // Vanished values are not finite
    /// let vanished = ScalarF5E4::MIN_POS / 28;
    /// assert!(!vanished.is_finite());
    ///
    /// // Undefined Scalars are not finite
    /// let undefined = ScalarF5E4::ZERO.pow(0);
    /// assert!(!undefined.is_finite());
    ///
    /// // Operations that produce normal results usually return finite values
    /// let still_finite = normal + 1;
    /// assert!(still_finite.is_finite());
    ///
    /// // Operations that exceed representable range escape finiteness
    /// let no_longer_finite = ScalarF5E4::MAX + ScalarF5E4::MAX / 2;
    /// assert!(!no_longer_finite.is_finite());
    ///
    /// // Infinity is definitely not finite!
    /// let infinity = ScalarF5E4::MAX / 0;
    /// assert!(!infinity.is_finite());
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
    /// // The one and only Zero
    /// let zero = Scalar::<i64, i16>::ZERO;
    /// assert!(zero.is_zero());
    ///
    /// // Even a very small number is not Zero
    /// let tiny = ScalarF6E4::MIN_POS / 61;
    /// assert!(!tiny.is_zero());
    /// assert!(tiny.vanished());  // It's vanished, not Zero
    ///
    /// // Normal values are not Zero
    /// let normal = ScalarF6E4::from(42);
    /// assert!(!normal.is_zero());
    ///
    /// // Exploded values are not Zero
    /// let huge = ScalarF6E4::MAX * ScalarF6E4::MAX;
    /// assert!(!huge.is_zero());
    ///
    /// // Infinity is not Zero (it's the opposite!)
    /// let infinity = ScalarF6E4::ONE / 0;
    /// assert!(!infinity.is_zero());
    ///
    /// // Multiplication by Zero always yields Zero
    /// let still_zero = zero * ScalarF6E4::PI;
    /// assert!(still_zero.is_zero());
    /// // Except with Infinity
    /// let undefined = zero * ScalarF6E4::INFINITY;
    /// assert!(!undefined.is_zero());
    ///
    /// // Reciprocal of infinity is Zero
    /// let also_zero = ScalarF6E4::ONE / infinity;
    /// assert!(also_zero.is_zero());
    ///
    /// // Adding a normal Scalar to Zero or vanished gives a normal Scalar
    /// let normal_again = zero + 163 + tiny + also_zero;
    /// assert!(!normal_again.is_zero());
    /// assert!(normal_again.is_normal());
    /// assert!(normal_again, 163);
    /// ```
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.exponent == E::AMBIGUOUS_EXPONENT && self.fraction == F::ZERO
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
    /// // Division by zero produces infinity
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!(infinity.is_infinite());
    ///
    /// // Exploded values are not infinity
    /// let exploded = ScalarF5E3::MAX * ScalarF5E3::MAX;
    /// assert!(!exploded.is_infinite());
    /// assert!(exploded.exploded());
    ///
    /// // Both are transfinite
    /// assert!(infinity.is_transfinite());
    /// assert!(exploded.is_transfinite());
    ///
    /// // Normal values are not infinity
    /// let normal = ScalarF5E3::from(42);
    /// assert!(!normal.is_infinite());
    ///
    /// // Zero is not infinity (it's the opposite!)
    /// let zero = ScalarF5E3::ZERO;
    /// assert!(!zero.is_infinite());
    ///
    /// // Reciprocal of infinity is exactly zero
    /// let zero_again = ScalarF5E3::ONE / infinity;
    /// assert!(zero_again.is_zero());
    ///
    /// // Infinity multiplied by any non-zero value remains infinity
    /// let still_infinity = infinity * ScalarF5E3::PI;
    /// assert!(still_infinity.is_infinite());
    ///
    /// // Infinity divided by any non-zero value remains infinity
    /// let still_infinity = infinity / 1000;
    /// assert!(still_infinity.is_infinite());
    ///
    /// // infinity + infinity is undefined, not infinity
    /// let undefined = infinity + infinity;
    /// assert!(!undefined.is_infinite());
    /// assert!(undefined.is_undefined());
    /// ```
    #[inline]
    pub fn is_infinite(&self) -> bool {
        self.exponent == E::AMBIGUOUS_EXPONENT && self.fraction == F::NEG_ONE
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
    /// // Normal positive numbers
    /// let positive = Scalar::<i64, i32>::from(42);
    /// assert!(positive.is_positive());
    ///
    /// // Zero is not positive
    /// let zero = ScalarF6E5::ZERO;
    /// assert!(!zero.is_positive());
    ///
    /// // Negative numbers are not positive
    /// let negative = ScalarF6E5::from(-37);
    /// assert!(!negative.is_positive());
    ///
    /// // Positive vanished values are positive
    /// let tiny = ScalarF6E5::MIN_POS / 131071;
    /// assert!(tiny.is_positive());
    ///
    /// // Positive exploded values are positive
    /// let huge = positive.pow(87539319);
    /// assert!(huge.is_positive());
    ///
    /// // Infinities aren't positive
    /// let infinity = ScalarF6E5::TAU / 0;
    /// assert!(!infinity.is_positive());
    ///
    /// // Neither are undefined Scalars
    /// let undefined = ScalarF6E5::ZERO / 0;
    /// assert!(!undefined.is_positive());
    /// ```
    #[inline]
    pub fn is_positive(&self) -> bool {
        // Extract high byte and cast to signed to use arithmetic shifts
        let prefix = self.prefix();

        // Check for actual Zero or undefined by shifting:
        // □□□□□□□□ -5-> □□□□□□□□ - Zero               0 - False
        // ■■■■■■■■ -5-> ■■■■■■■■ - Infinity           ∞ - False
        // □■xxxxxx -5-> □□□□□□■x - Normal numbers    +1 - False
        // ■□xxxxxx -5-> ■■■■■■□x - Negative numbers  -1 - True
        // □■xxxxxx -5-> □□□□□□■x - Positive exploded +↑ - False
        // ■□xxxxxx -5-> ■■■■■■□x - Negative exploded -↑ - True
        // □□■xxxxx -5-> □□□□□□□■ - Positive vanished +↓ - False
        // ■■□xxxxx -5-> ■■■■■■■□ - Negative vanished -↓ - True
        // □□□■xxxx -5-> □□□□□□□□ - Undefined         ℘ - False
        // ■■■□xxxx -5-> ■■■■■■■■ - Undefined         ℘ - False
        let top_three = prefix >> 5;
        if top_three == top_three.rotate_right(1) {
            return false;
        }

        // Check sign bit without zero equality
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
    /// // Normal negative numbers
    /// let negative = Scalar::<i64, i32>::from(-42);
    /// assert!(negative.is_negative());
    ///
    /// // Zero is not negative
    /// let zero = ScalarF6E5::ZERO;
    /// assert!(!zero.is_negative());
    ///
    /// // Positive numbers are not negative
    /// let positive = ScalarF6E5::from(37);
    /// assert!(!positive.is_negative());
    ///
    /// // Negative vanished values are negative
    /// let tiny = ScalarF6E5::MIN_POS / -131071;
    /// assert!(tiny.is_negative());
    ///
    /// // Negative exploded values are negative
    /// let huge = negative.pow(87539319);
    /// assert!(huge.is_negative());
    ///
    /// // Infinities aren't negative
    /// let infinity = ScalarF6E5::TAU / 0;
    /// assert!(!infinity.is_negative());
    ///
    /// // Neither are undefined Scalars
    /// let undefined = ScalarF6E5::ZERO / 0;
    /// assert!(!undefined.is_negative());
    /// ```
    #[inline]
    pub fn is_negative(&self) -> bool {
        // Extract high byte and cast to signed to use arithmetic shifts
        let prefix = self.prefix();

        // Check for Zero or undefined by shifting:
        // □□□□□□□□ -5-> □□□□□□□□ - Zero               0 - False
        // ■■■■■■■■ -5-> ■■■■■■■■ - Infinity           ∞ - False
        // □■xxxxxx -5-> □□□□□□■x - Normal numbers    +1 - False
        // ■□xxxxxx -5-> ■■■■■■□x - Negative numbers  -1 - True
        // □■xxxxxx -5-> □□□□□□■x - Positive exploded +↑ - False
        // ■□xxxxxx -5-> ■■■■■■□x - Negative exploded -↑ - True
        // □□■xxxxx -5-> □□□□□□□■ - Positive vanished +↓ - False
        // ■■□xxxxx -5-> ■■■■■■■□ - Negative vanished -↓ - True
        // □□□■xxxx -5-> □□□□□□□□ - Undefined         ℘ - False
        // ■■■□xxxx -5-> ■■■■■■■■ - Undefined         ℘ - False
        let top_three = prefix >> 5;
        if top_three == top_three.rotate_right(1) {
            return false;
        }

        // Check sign bit
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
    /// // The answer to the meaning of life is an integer
    /// let integer = Scalar::<i8, i16>::from(42);
    /// assert!(integer.is_integer());
    ///
    /// // Zero is an integer
    /// let zero = ScalarF3E4::ZERO;
    /// assert!(zero.is_integer());
    ///
    /// // Negative integers are still integers
    /// let neg_int = ScalarF3E4::from(-7);
    /// assert!(neg_int.is_integer());
    ///
    /// // Numbers with fractional values are not integers
    /// let fract = ScalarF3E4::from(1.122757);
    /// assert!(!fract.is_integer());
    ///
    /// // Exploded values are considered integers
    /// let huge = ScalarF3E4::MAX.pow(15);
    /// assert!(huge.is_integer());
    ///
    /// // Infinity is not an integer
    /// let infinity = ScalarF3E4::ONE / 0;
    /// assert!(infinity.is_integer());
    ///
    /// // Vanished values are not integers
    /// let tiny = ScalarF3E4::MIN_POS / 3435;
    /// assert!(!tiny.is_integer());
    ///
    /// // Undefined states are not integers
    /// let undefined = ScalarF3E4::ZERO / 0;
    /// assert!(!undefined.is_integer());
    /// ```
    #[inline]
    pub fn is_integer(&self) -> bool {
        // Case 0: Exponent is >= FRACTION_BITS, which means the value is entirely in the integer portion, I think? do check for -'s :)
        if self.exponent >= (F::FRACTION_BITS.wrapping_sub(1)).as_() {
            return true;
        }
        // Case 1: Negative exponent means we have a fraction only, undefined, Infinity, Zero or escaped
        if self.exponent.is_negative() {
            if self.exponent == E::AMBIGUOUS_EXPONENT {
                let prefix = self.prefix();
                if prefix == 0 {
                    //Zero is an integer
                    return true;
                }

                // Check for N-1 exploded patterns by shifting:
                // □■xxxxxx -6-> □□□□□□□■
                // ■□xxxxxx -6-> ■■■■■■■□
                let top_two = prefix >> 6;
                return top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8;
            }
            return false;
        }
        // Case 2: Check fractional bits in left-aligned format
        let exponent_usize: isize = self.exponent.as_();
        let frac_bits = F::FRACTION_BITS.wrapping_sub(exponent_usize);

        // If no fractional bits, it's definitely an integer
        if frac_bits <= 0 {
            return true;
        }

        // For left-aligned format with unbiased exponents:
        // Check if fraction << (exponent + 1) == 0
        // This shifts out the integer part, leaving only fractional bits
        let exp_isize: isize = self.exponent.as_();
        let shift_amount = exp_isize.wrapping_add(1);

        if shift_amount >= F::FRACTION_BITS {
            // Large exponent means no fractional bits possible
            return true;
        }

        if shift_amount <= 0 {
            // Small/negative exponent, handle via earlier cases
            return false;
        }

        // Shift left by (exponent + 1) and check if result is zero
        (self.fraction << shift_amount) == F::ZERO
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
    /// // Integers within contiguous range
    /// let small_int = ScalarF4E3::from(42);
    /// assert!(small_int.is_contiguous());
    ///
    /// // Zero is within the contiguous range
    /// let zero = ScalarF4E3::ZERO;
    /// assert!(zero.is_contiguous());
    ///
    /// // Large integers outside contiguous range return false,
    /// // even tho they are precisely representable
    /// let large_int = Scalar::<i16, i8>::from(1) << 15;
    /// assert!(!large_int.is_contiguous());
    ///
    /// // Fractional values are not contiguous integers
    /// let fract = ScalarF4E3::PI;
    /// assert!(!fract.is_contiguous());
    ///
    /// // Vanished values and infinite states are never contiguous
    /// let tiny = ScalarF4E3::MIN_POS / ScalarF4E3::MAX;
    /// assert!(!tiny.is_contiguous());
    /// let infinity = ScalarF4E3::ONE / 0;
    /// assert!(!infinity.is_contiguous());
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

        // For normal numbers with non-negative exponents
        if self.is_normal() && !self.exponent.is_negative() {
            let exp_isize: isize = self.exponent.saturate();

            // If exponent is >= FRACTION_BITS - 1, the number is too large to fit
            // in the contiguous integer range (since we need some bits for the fractional part)
            if exp_isize >= F::FRACTION_BITS.wrapping_sub(1).as_() {
                return false;
            }

            // For contiguous integers, we need the value to be representable exactly
            // This is already guaranteed by is_integer(), so we just need to check
            // if it's within the contiguous range. For left-aligned format,
            // contiguous integers should have reasonable magnitudes.
            return true;
        }

        // Handle special cases (exploded/vanished integers)
        if self.exponent.is_negative() && self.exponent == E::AMBIGUOUS_EXPONENT {
            let prefix = self.prefix();
            // Check for exploded integer patterns
            let top_two = prefix >> 6;
            return top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8;
        }

        false
    }

    pub(crate) fn scalar_negate(&mut self) {
        if !self.is_normal() {
            // Check if top 3 bits are equal by pushing 5 bits off
            // ↓↓↓                ↓↓↓
            // □□□xxxxx -5-> □□□□□□□□ - Undefined (℘), Zero (0), infinity (∞)
            let top_three = self.prefix() >> 5;
            // Then rotate and compare.  If uniform, they will be the same
            if top_three == top_three.rotate_right(1) {
                return;
            }
            if self.fraction == F::POS_ONE_FRACTION {
                self.fraction = F::NEG_ONE_FRACTION;
            } else if self.fraction == F::NEG_ONE_FRACTION {
                self.fraction = F::POS_ONE_FRACTION;
            } else if self.fraction == F::POS_SMALL_FRACTION {
                self.fraction = F::NEG_SMALL_FRACTION;
            } else if self.fraction == F::NEG_SMALL_FRACTION {
                self.fraction = F::POS_SMALL_FRACTION;
            } else {
                self.fraction = self.fraction.wrapping_neg();
            }
            return;
        }
        if self.fraction == F::POS_ONE_FRACTION {
            self.exponent = self.exponent.wrapping_sub(&E::ONE);
            if self.exponent == E::AMBIGUOUS_EXPONENT {
                self.fraction = F::NEG_SMALL_FRACTION;
            } else {
                self.fraction = F::NEG_ONE_FRACTION;
            }
        } else if self.fraction == F::NEG_ONE_FRACTION {
            self.fraction = F::POS_ONE_FRACTION;
            self.exponent = self.exponent.wrapping_add(&E::ONE);
        } else {
            self.fraction = self.fraction.wrapping_neg();
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
    /// // Magnitude of negative is positive
    /// let negative = Scalar::<i16, i64>::from(-42);
    /// assert!(negative.magnitude() == ScalarF4E6::from(42));
    ///
    /// // Magnitude of positive remains the same
    /// let positive = ScalarF4E6::from(123);
    /// assert!(positive.magnitude() == positive);
    ///
    /// // Magnitude of Zero is Zero
    /// let zero = ScalarF4E6::ZERO;
    /// assert!(zero.magnitude() == zero);
    ///
    /// // Magnitude preserves scale of vanished values
    /// let tiny_neg = ScalarF4E6::MIN_POS / -28;
    /// assert!(tiny_neg.magnitude().is_positive());
    /// assert!(tiny_neg.magnitude().vanished());
    ///
    /// // Magnitude preserves scale of exploded values
    /// let huge_neg = ScalarF4E6::MIN * ScalarF4E6::MAX;
    /// assert!(huge_neg.magnitude().is_positive());
    /// assert!(huge_neg.magnitude().exploded());
    ///
    /// // Undefined states pass thru magnitude() unchanged
    /// let undefined = ScalarF4E6::ZERO / 0;
    /// assert!(undefined.magnitude().is_undefined());
    /// ```
    pub fn magnitude(&self) -> Self {
        if !self.fraction.is_negative() {
            return *self;
        }
        -self
    }

    /// Returns the normalized unit form of this Scalar `[+1]` or `[-1]`
    ///
    /// # Description
    ///
    /// Creates a unit Scalar (magnitude 1) with the same sign as this Scalar.
    /// This preserves the orientation while normalizing the magnitude.
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
    /// // Sign of a positive number is 1
    /// let positive = Scalar::<i32, i8>::from(42);
    /// assert!(positive.sign() == 1);
    ///
    /// // Sign of a negative number is -1
    /// let negative = ScalarF5E3::from(-3.14);
    /// assert!(negative.sign() == -1);
    ///
    /// // Sign of Zero is undefined (no direction)
    /// let zero = ScalarF5E3::ZERO;
    /// assert!(zero.sign().is_undefined());
    ///
    /// // Sign of Infinity is undefined (no direction)
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!(infinity.sign().is_undefined());
    ///
    /// // Sign of undefined remains unchanged
    /// let undefined = 0 / ScalarF5E3::ZERO; // 0/0
    /// assert!(undefined.sign().is_undefined());
    /// ```
    pub fn sign(&self) -> Self {
        if self.is_undefined() {
            return *self;
        }
        if self.is_n0() {
            return Self {
                fraction: SIGN_INDETERMINATE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
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
    /// // The floor of the answer to everything is itself
    /// let integer = Scalar::<i32, i32>::from(42);
    /// assert!(integer.floor() == integer);
    ///
    /// // Floor of positive fractional number
    /// let fract_pos = ScalarF5E5::from(2.622057554);
    /// assert!(fract_pos.floor() == 2);
    ///
    /// // Floor of negative fractional number
    /// let fract_neg = ScalarF5E5::from(-24.1);
    /// assert!(fract_neg.floor() == -25);
    ///
    /// // Floor of Zero is Zero
    /// let zero = ScalarF5E5::ZERO;
    /// assert!(zero.floor() == 0);
    ///
    /// // Floor of positive vanished is Zero
    /// let tiny_pos = ScalarF5E5::MIN_POS / 12;
    /// assert!(tiny_pos.floor() == 0);
    ///
    /// // Floor of negative vanished is negative one
    /// let tiny_neg = ScalarF5E5::MIN_POS / -24;
    /// assert!(tiny_neg.floor() == -1);
    ///
    /// // Floor of exploded preserves the value
    /// let sploded = ScalarF5E5::MAX * ScalarF5E5::MAX;
    /// assert!(sploded.floor() == sploded);
    ///
    /// // Floor of Infinity preserves Infinity
    /// let infinity = ScalarF5E5::ONE / 0;
    /// assert!(infinity.floor() == infinity);
    ///
    /// // Floor of undefined remains undefined
    /// let undefined = ScalarF5E5::ZERO / 0;
    /// assert!(undefined.floor().is_undefined());
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

        if self.exponent <= E::ZERO {
            if self.is_negative() {
                let result = Self::NEG_ONE;
                return result;
            }
            let result = Self::ZERO;
            return result;
        }
        let mut result = *self;
        let width = F::FRACTION_BITS.wrapping_sub(1);
        if result.exponent >= width.as_() {
            return result;
        }
        let e: isize = result.exponent.as_();
        let frac_bits = width.wrapping_sub(e);
        let mask: F = !((F::ONE << frac_bits).wrapping_sub(&F::ONE));
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
    /// // The ceiling of an integer is itself
    /// let integer = Scalar::<i128, i32>::from(42);
    /// assert!(integer.ceil() == integer);
    ///
    /// // Ceiling of positive fractional number
    /// let fract_pos = ScalarF7E5::E.pow(ScalarF7E5::PI);
    /// assert!(fract_pos.ceil() == 24);
    ///
    /// // Ceiling of negative fractional number
    /// let fract_neg = ScalarF7E5::from(-24.1);
    /// assert!(fract_neg.ceil() == -24);
    ///
    /// // Ceiling of Zero is Zero
    /// let zero = ScalarF7E5::ZERO;
    /// assert!(zero.ceil() == 0);
    ///
    /// // Ceiling of positive vanished is one
    /// let tiny_pos = ScalarF7E5::MIN_POS / 17;
    /// assert!(tiny_pos.ceil() == 1);
    ///
    /// // Ceiling of negative vanished is Zero
    /// let tiny_neg = ScalarF7E5::MAX_NEG / 29;
    /// assert!(tiny_neg.ceil() == 0);
    ///
    /// // Ceiling of exploded preserves the value
    /// let sploded = ScalarF7E5::MAX * ScalarF7E5::MAX;
    /// assert!(sploded.ceil() == sploded);
    ///
    /// // Ceiling of Infinity preserves Infinity
    /// let infinity = ScalarF7E5::ONE / 0;  // Division by zero produces Infinity
    /// assert!(infinity.ceil() == infinity);
    ///
    /// // Ceiling of undefined remains undefined
    /// let undefined = ScalarF7E5::ZERO / 0;  // 0/0 is undefined
    /// assert!(undefined.ceil().is_undefined());
    /// ```
    pub fn ceil(&self) -> Self {
        -(-*self).floor()
    }

    /// Returns the nearest integer Scalar
    ///
    /// # Description
    ///
    /// Rounds this Scalar to the nearest integer value using banker's rounding.
    /// Ties (exactly integer + 1/2) are rounded to the nearest even integer.
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
    /// // Rounding an integer returns the same integer
    /// let integer = Scalar::<i128, i64>::from(42);
    /// assert!(integer.round() == integer);
    ///
    /// // Rounding positive numbers
    /// let e_m = ScalarF7E6::from(0.57721566490153286060651209008240243104);
    /// assert!(e_m.round() == 1);
    ///
    /// let tau = ScalarF7E6::TAU;
    /// assert!(tau.round() == 6);
    ///
    /// // Ties are rounded up
    /// let half = ScalarF7E6::ONE / 2;
    /// assert!(half.round() == 1);
    /// assert!((-half).round() == 0);
    ///
    /// // Rounding negative numbers
    /// let series = ScalarF7E6::NEG_ONE / 12;
    /// assert!(series.round() == 0);
    ///
    /// // Rounding Zero returns Zero
    /// let zero = ScalarF7E6::ZERO;
    /// assert!(zero.round() == 0);
    ///
    /// // Vanished values round to Zero
    /// let tiny = ScalarF7E6::MIN_POS / 17;
    /// assert!(tiny.round() == 0);
    ///
    /// // Exploded values remain unchanged
    /// let huge = ScalarF7E6::MAX * ScalarF7E6::MAX;
    /// assert!(huge.round().fraction == huge.fraction && huge.round().exponent == huge.exponent);
    ///
    /// // Infinity remains unchanged
    /// let infinity = ScalarF7E6::ONE / 0;
    /// assert!(infinity.round().fraction == infinity.fraction && infinity.round().exponent == infinity.exponent);
    ///
    /// // Undefined Scalars maintain their undefined state
    /// let undefined = ScalarF7E6::ZERO / 0;
    /// assert!(undefined.round().fraction == undefined.fraction && undefined.round().exponent == undefined.exponent);
    /// ```
    pub fn round(&self) -> Self {
        // Handle non-normal values first
        if !self.is_normal() {
            if self.vanished() {
                return Self::ZERO;
            }
            return *self;
        }

        // Use math approach for banker's rounding (will optimize to bits later)
        let floored = self.floor();
        let frac = *self - floored;

        // Check if fractional part is exactly 0.5
        if (frac - Self::HALF).is_zero() {
            // Banker's rounding: round to even integer
            // For banker's rounding, we need to check which of the two nearest integers is even
            let lower = floored;
            let upper = floored + Self::ONE;

            let lower_is_even = (lower % Self::TWO).is_zero();
            let upper_is_even = (upper % Self::TWO).is_zero();

            if lower_is_even {
                return lower;
            } else if upper_is_even {
                return upper;
            } else {
                // This happens if we were out of the contiguous range, thus indicating all numbers are now even
                return floored;
            }
        } else if frac > Self::HALF {
            // Round up (away from zero)
            return floored + Self::ONE;
        } else {
            // Round down (toward zero)
            return floored;
        }
    }

    /// Returns the fractional part of this Scalar
    ///
    /// # Description
    ///
    /// Extracts the fractional part of this Scalar by subtracting its floor, following the mathematical definition: frac(x) = x - ⌊x⌋
    /// This returns a value in the range [0,1).
    /// Future implementations will utilize bit masking and normalization
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
    /// // Fractional part of an integer is Zero
    /// let integer = Scalar::<i128, i16>::from(42);
    /// assert!(integer.frac() == 0);
    ///
    /// // Fractional part of positive number
    /// let pi = ScalarF7E4::PI;
    /// assert!(pi - pi.frac() == pi.floor());
    /// assert!(pi.frac() > 0 && pi.frac() < 1); // ~0.14159...
    ///
    /// // Fractional part of negative number
    /// let neg = ScalarF7E4::from(-3.25);
    /// assert!(neg.frac() == 0.75);
    ///
    /// // Fractional part of Zero is Zero
    /// let zero = ScalarF7E4::ZERO;
    /// assert!(zero.frac() == 0);
    ///
    /// // Fractional part of positive vanished returns the value itself
    /// let tiny_pos = ScalarF7E4::MIN_POS / 12;
    /// assert!(tiny_pos.frac().vanished() && tiny_pos.frac().is_positive());
    ///
    /// // Fractional part of negative vanished approaches 1
    /// let tiny_neg = ScalarF7E4::MIN_POS / -67;
    /// assert!(tiny_neg.frac() - ScalarF7E4::EFFECTIVELY_POS_ONE == 0);
    ///
    /// // Fractional part of exploded values is Zero
    /// let huge = ScalarF7E4::MAX.square();
    /// assert!(huge.frac() == ScalarF7E4::ZERO);
    ///
    /// // Fractional part of Infinity is undefined
    /// let infinity = ScalarF7E4::ONE / 0;
    /// assert!(infinity.frac().is_undefined());
    ///
    /// // Fractional part of undefined remains undefined
    /// let undefined = ScalarF7E4::ZERO / 0;
    /// assert!(undefined.frac().is_undefined());
    /// ```
    pub fn frac(&self) -> Self {
        if !self.is_normal() {
            // Handle vanished values
            if self.vanished() && self.is_negative() {
                // Negative vanished - almost one
                return Self::EFFECTIVELY_POS_ONE;
            }
            if self.exploded() {
                return Self::ZERO;
            }
            if self.is_infinite() {
                return Self {
                    fraction: FRACTIONAL_INFINITY.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        if self.exponent < E::ZERO {
            if self.is_negative() {
                // Align to exp=0 by sign-extend shift, then fall through to mask
                let mut result = *self;
                let shift: isize = (E::ZERO - result.exponent).as_();
                let shift: usize = shift.max(0) as usize;
                let max_shift = (F::FRACTION_BITS - 1) as usize;
                result.fraction = result.fraction >> shift.min(max_shift);
                result.exponent = E::ZERO;
                let frac_bits = max_shift;
                let mask: F = (F::ONE << frac_bits).wrapping_sub(&F::ONE);
                result.fraction = result.fraction & mask;
                result.normalize();
                return result;
            }
            return *self;
        }
        let width = F::FRACTION_BITS.wrapping_sub(1);
        if self.exponent >= width.as_() {
            // Already an integer, no fractional part
            return Self::ZERO;
        }
        let mut result = *self;
        let e: isize = result.exponent.as_();
        let frac_bits = width.wrapping_sub(e);
        let mask: F = (F::ONE << frac_bits).wrapping_sub(&F::ONE);
        result.fraction = result.fraction & mask;
        result.normalize();
        result
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
    /// - `[-↑]` Negative exploded (least)
    /// -
    /// - `[℘?]` or `[∞]` Unordered!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E6};
    ///
    /// // Normal values
    /// let a = Scalar::<i64, i64>::from(42);
    /// let b = ScalarF6E6::from(21);
    /// assert!(a.max(b) == a);
    ///
    /// // With negative values
    /// let neg = ScalarF6E6::from(-12);
    /// assert!(neg.max(b) == b);
    ///
    /// // With Zero
    /// let zero = ScalarF6E6::ZERO;
    /// assert!(neg.max(zero) == zero);
    ///
    /// // With vanished values
    /// let tiny_pos = ScalarF6E6::MIN_POS / 4;
    /// assert!(tiny_pos.max(neg).is_positive());
    ///
    /// // With exploded values
    /// let huge = ScalarF6E6::MIN * ScalarF6E6::MAX;
    /// assert!(a.max(5) == 5);
    ///
    /// // With Infinity (mathematically undefined comparison)
    /// let infinity = 1 / zero;
    /// assert!(a.max(infinity).is_undefined());
    /// assert!(infinity.max(a).is_undefined());
    ///
    /// // Vanished values with same sign are not comparable
    /// let little = ScalarF6E6::MIN_POS.square();
    /// assert!(tiny_pos.max(little).is_undefined());
    ///
    /// // With undefined
    /// let undefined = infinity + 1;
    /// assert!(a.max(undefined).is_undefined());
    /// assert!(undefined.max(a).is_undefined());
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.vanished()
                && other.vanished()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MAX_UNORDERED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.exploded()
                && other.exploded()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MAX_UNORDERED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
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
    /// - `[-↑]` Negative exploded (least)
    /// -
    /// - `[℘?]` or `[∞]` Unordered!
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E6};
    ///
    /// // Normal values
    /// let a = Scalar::<i32, i64>::from(42); // Using turbofish notation
    /// let b = ScalarF5E6::from(21); // Using shorthand
    /// assert!(a.min(b) == b); // Using owned value
    /// assert!(a.min(&b) == b); // Using reference
    ///
    /// // With negative values
    /// let neg = ScalarF5E6::from(-42);
    /// assert!(a.min(neg) == neg);
    ///
    /// // With Zero
    /// assert!(neg.min(0) == neg);
    ///
    /// // With Infinity (mathematically undefined comparison)
    /// let infinity = ScalarF5E6::ONE / 0;
    /// assert!(a.min(infinity).is_undefined());
    /// assert!(infinity.min(a).is_undefined());
    ///
    /// // With vanished values
    /// let vanished_neg = ScalarF5E6::MIN_POS / -16;
    /// assert!(vanished_neg.min(a).is_negative());
    ///
    /// // With exploded values
    /// let exploded = ScalarF5E6::MAX / 0.0625;
    /// assert!(a.min(exploded) == a);
    ///
    /// // Vanished values with same sign are not comparable
    /// let small = ScalarF5E6::MAX_NEG / 512;
    /// assert!(vanished_neg.min(small).is_undefined());
    ///
    /// // With undefined
    /// let undefined = ScalarF5E6::ZERO / 0;  // 0/0 is undefined
    /// assert!(a.min(undefined).is_undefined());
    /// assert!(undefined.min(a).is_undefined());
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.vanished()
                && other.vanished()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MIN_UNORDERED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.exploded()
                && other.exploded()
                && ((self.is_positive() && other.is_positive())
                    || (self.is_negative() && other.is_negative()))
            {
                return Self {
                    fraction: MIN_UNORDERED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
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
    /// // Normal value clamping with Scalar bounds
    /// let value = ScalarF4E3::from(42);
    /// let min = ScalarF4E3::from(8);
    /// let max = ScalarF4E3::from(32);
    /// assert!(value.clamp(min, max) == max);
    ///
    /// // With mixed primitive types
    /// let value2 = ScalarF4E3::from(42);
    /// assert!(value2.clamp(8i16, 32f32) == ScalarF4E3::from(32));
    /// assert!(value2.clamp(50u8, 100i64) == value2); // within range
    ///
    /// // Value within range
    /// let in_range = ScalarF4E3::from(16);
    /// assert!(in_range.clamp(min, max) == in_range);
    ///
    /// // Value below range
    /// let below = ScalarF4E3::from(4);
    /// assert!(below.clamp(min, max) == min);
    ///
    /// // With Infinity (mathematically undefined)
    /// let infinity = ScalarF4E3::ONE / 0;
    /// assert!(value.clamp(min, infinity).is_undefined());
    /// assert!(value.clamp(infinity, max).is_undefined());
    /// assert!(infinity.clamp(min, max).is_undefined());
    ///
    /// // With special values
    /// let zero = ScalarF4E3::ZERO;
    /// let tiny_pos = ScalarF4E3::MIN_POS / 64;
    /// let neg = ScalarF4E3::from(-16);
    ///
    /// // Clamping between negative and positive
    /// assert!(neg.clamp(neg, zero) == neg);
    /// assert!(tiny_pos.clamp(neg, zero) == tiny_pos);
    ///
    /// // With undefined
    /// let undefined = ScalarF4E3::ZERO / 0; // 0/0 undefined state
    /// assert!(value.clamp(min, &undefined).is_undefined());
    /// assert!(undefined.clamp(&min, max).is_undefined());
    ///
    /// // Incomparable bounds with vanished values
    /// let miniscule_positive = ScalarF4E3::MIN_POS / 128;
    /// assert!(value.clamp(tiny_pos, miniscule_positive).is_undefined());
    ///
    /// // Unordered bounds
    /// assert!(value.clamp(max, min).is_undefined());
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
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if min > max {
            return Self {
                fraction: CLAMP_UNORDERED.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let clamped = self.max(min).min(max);
        if clamped.is_undefined() {
            return Self {
                fraction: CLAMP_UNORDERED.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
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
    /// assert!(!Scalar::<i128,i8>::ZERO.is_prime());   // 0 is not prime
    /// assert!(!ScalarF7E3::ONE.is_prime());           // 1 is not prime
    /// assert!(ScalarF7E3::TWO.is_prime());            // 2 is prime
    /// assert!(ScalarF7E3::from(3).is_prime());        // 3 is prime
    /// assert!(!ScalarF7E3::from(4).is_prime());       // 4 is not prime
    /// assert!(ScalarF7E3::from(17).is_prime());       // 17 is prime
    /// assert!(!ScalarF7E3::from(68).is_prime());      // 68 is not prime
    ///
    /// // Negative numbers are not prime
    /// assert!(!ScalarF7E3::NEG_ONE.is_prime());
    /// assert!(!ScalarF7E3::from(-7).is_prime());
    ///
    /// // Non-integer values are not prime
    /// assert!(!ScalarF7E3::PI.is_prime());
    /// assert!(!ScalarF7E3::from(2.5).is_prime());
    /// ```
    #[inline]
    pub fn is_prime(&self) -> bool {
        if self.exponent < 2.as_() || self.exponent >= (F::FRACTION_BITS - 1).as_() {
            return false;
        }

        if self.fraction.is_negative() {
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

    /// Normalizes this Scalar by shifting the fraction left until the most significant bit is in the N-1 position, adjusting the exponent accordingly.  
    /// Sign is placed in N-0.  
    ///  
    /// If shifting would cause a small number to vanish, marks the number as ambiguous and normalizes it to N-2.  
    ///  
    /// 01234567...  
    ///  
    /// □■xxxxxx... - Normal positive numbers  
    ///  
    /// ■□xxxxxx... - Normal negative numbers  
    ///  
    /// - Normalizes all scalars!
    #[inline]
    pub fn normalize(&mut self) {
        let shift = self
            .fraction
            .leading_ones()
            .max(self.fraction.leading_zeros());
        if shift > 1 {
            let shift = shift as isize;
            let new_exponent;
            if shift == F::FRACTION_BITS {
                if !self.fraction.is_negative() {
                    self.exponent = E::AMBIGUOUS_EXPONENT;
                    return;
                }
                new_exponent = self.exponent.wrapping_sub(&(shift.wrapping_add(1)).as_());
            } else {
                new_exponent = self.exponent.wrapping_sub(&shift.as_());
            }

            if self.exponent.is_negative() && !new_exponent.is_negative() {
                self.exponent = E::AMBIGUOUS_EXPONENT;
                self.fraction = self.fraction << (shift.wrapping_sub(2));
            } else {
                self.exponent = new_exponent.wrapping_add(&E::ONE);
                self.fraction = self.fraction << (shift.wrapping_sub(1));
            }
        }
    }
    #[inline]
    /// Normalizes an exploded Scalar by shifting the fraction left until the most significant bit is in the N-1 position, exponent is not touched.  
    ///  
    /// Example bit positions:
    /// 01234567...
    /// □■xxxxxx... - Exploded positive numbers  
    /// ■□xxxxxx... - Exploded negative numbers  
    pub(crate) fn normalize_exploded(&mut self) {
        let shift = self
            .fraction
            .leading_ones()
            .max(self.fraction.leading_zeros());
        self.fraction = self.fraction << (shift.wrapping_sub(1) as usize);
    }

    #[inline]
    /// Normalizes a vanished Scalar by shifting its fraction to the N-2 position.  
    /// Sign bits occupy N-0 and N-1, exponent is not touched
    ///  
    /// Example bit positions:
    /// 01234567...  
    /// □□■xxxxx... - Vanished positive numbers  
    /// ■■□xxxxx... - Vanished negative numbers  
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
    let bits = core::mem::size_of::<T>().wrapping_mul(8);
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
