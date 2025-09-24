use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use std::ops::{Shl, Shr};
macro_rules! impl_circle_new {
    ($($f:ty, $e:ty);*) => {
        $(
impl Circle<$f, $e> {
    /// Creates a new Circle from raw real, imaginary and exponent integers.
    ///
    /// This is a low-level constructor that directly sets the internal state.
    /// For normal number creation, use `from()` which handles:
    /// - Proper normalization of components
    /// - Exponent adjustment
    /// - Special value handling
    ///
    /// ```
    /// # use spirix::Circle;
    /// let raw_circle = Circle::<i32, i8>::new(0b10101 << 26, 0, 6);
    /// let normal = CircleF5E3::from(42);
    /// assert!(raw_circle == normal);
    /// ```
    #[inline]
    pub fn new(real: $f, imaginary: $f, exponent: $e) -> Circle<$f, $e> {
        Circle { real, imaginary, exponent }
    }
}
        )*
    }
}
impl_circle_new! {
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
            + WrappingSub
            + WrappingMul,
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
            + WrappingSub
            + WrappingMul,
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
    /// Returns the real part of this Circle as a Scalar
    ///
    /// # Description
    ///
    /// Extracts the real component of this Circle as a Scalar value, preserving state information.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[#]` Real component as a normal Scalar
    /// - `[0]` ➔ `[0]` Zero
    /// - `[↑]` ➔ `[↑]` Real part as an exploded Scalar
    /// - `[↓]` ➔ `[↓]` Real part as a vanished Scalar
    /// - `[∞]` ➔ `[∞]` Infinity
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Real part of a normal complex number
    /// let complex = Circle::<i32, i8>::from((3.14, 2.71));
    /// assert!(complex.r() == 3.14);
    ///
    /// // Real part of Zero is Zero
    /// let zero = CircleF5E3::ZERO;
    /// assert!(zero.r().is_zero());
    ///
    /// // Real part of exploded preserves state
    /// let exploded = CircleF5E3::MAX_REAL * 2;
    /// assert!(exploded.r().exploded());
    ///
    /// // Real part of undefined is undefined
    /// let undefined = zero / 0;
    /// assert!(undefined.r().is_undefined());
    /// ```
    #[inline]
    pub fn r(&self) -> Scalar<F, E> {
        let mut scalar = Scalar {
            fraction: self.real,
            exponent: self.exponent,
        };
        if self.is_normal() {
            scalar.normalize();
        } else if self.vanished() {
            scalar.normalize_vanished();
        } else if self.exploded() {
            scalar.normalize_exploded();
        }
        scalar
    }

    /// Returns the imaginary part of this Circle as a Scalar
    ///
    /// # Description
    ///
    /// Extracts the imaginary component of this Circle as a Scalar, preserving state information.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[#]` Imaginary component as a normal Scalar
    /// - `[0]` ➔ `[0]` Zero
    /// - `[↑]` ➔ `[↑]` Imaginary part as an exploded Scalar
    /// - `[↓]` ➔ `[↓]` Imaginary part as a vanished Scalar
    /// - `[∞]` ➔ `[∞]` Infinity
    /// - `[℘?]` ➔ `[℘?]` The same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Imaginary part of a normal complex number
    /// let complex = Circle::<i32, i8>::from((3.14, 2.71));
    /// assert!(complex.i() == 2.71);
    ///
    /// // Imaginary part of Zero is Zero
    /// let zero = CircleF5E3::ZERO;
    /// assert!(zero.i().is_zero());
    ///
    /// // Imaginary part of exploded preserves state
    /// let huge = CircleF5E3::from((1, ScalarF5E3::MAX * 2));
    /// assert!(huge.i().exploded());
    ///
    /// // Imaginary part of undefined is undefined
    /// let undefined = zero / 0;
    /// assert!(undefined.i().is_undefined());
    /// ```
    #[inline]
    pub fn i(&self) -> Scalar<F, E> {
        let mut scalar = Scalar {
            fraction: self.imaginary,
            exponent: self.exponent,
        };
        if self.is_normal() {
            scalar.normalize();
        } else if self.vanished() {
            scalar.normalize_vanished();
        } else if self.exploded() {
            scalar.normalize_exploded();
        }
        scalar
    }

    /// Returns true if this Circle is a normal complex number
    ///
    /// # Description
    ///
    /// Normal complex numbers have definite magnitudes and orientations and participate fully in all arithmetic operations. Unlike Zero or escaped values, normal complex numbers occupy the "standard" region of numeric space where arithmetic behaves conventionally.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `true` Normal complex numbers
    /// - `[0]` ➔ `false` Zero
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF4E4};
    ///
    /// // Regular complex numbers are normal
    /// let normal = Circle::<i16, i16>::from((42, 13.5));
    /// assert!(normal.is_normal());
    ///
    /// // Normal values maintain their normality thru standard operations
    /// let still_normal = normal * CircleF4E4::from((1, 1)) / 2;
    /// assert!(still_normal.is_normal());
    ///
    /// // Zero is not normal
    /// let zero = CircleF4E4::ZERO;
    /// assert!(!zero.is_normal());
    ///
    /// // Exploded values are not normal
    /// let exploded = CircleF4E4::MAX * CircleF4E4::MAX;
    /// assert!(!exploded.is_normal());
    ///
    /// // Vanished values are not normal
    /// let vanished = CircleF4E4::MIN_POS / 45;
    /// assert!(!vanished.is_normal());
    ///
    /// // Undefined Circles are definitely not normal
    /// let undefined = CircleF4E4::ONE / 0;
    /// assert!(!undefined.is_normal());
    ///
    /// // Operations that exceed representable range escape normality
    /// let no_longer_normal = CircleF4E4::MAX_NEG.square();
    /// assert!(!no_longer_normal.is_normal());
    /// ```
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.exponent != E::AMBIGUOUS_EXPONENT
    }

    /// Checks if this Circle is in an undefined state `[℘?]`
    ///
    /// # Description
    ///
    /// Undefined states represent values from operations that have no known or agreed upon mathematical result. Spirix uses specific bit patterns to track various types of undefined states, maintaining "first cause" information.
    ///
    /// # Returns
    ///
    /// - `[℘?]` ➔ `true` Undefined states
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[0]` ➔ `false` Zero
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Normal complex numbers are defined
    /// let normal = CircleF5E3::from((42, 7));
    /// assert!(!normal.is_undefined());
    ///
    /// // Infinity is defined
    /// let infinity = normal / 0;
    /// assert!(!infinity.is_undefined());
    ///
    /// // Zero is defined
    /// let zero = CircleF5E3::ZERO;
    /// assert!(!zero.is_undefined());
    ///
    /// // Escaped values are defined
    /// let exploded = CircleF5E3::MAX * CircleF5E3::MIN;
    /// assert!(!exploded.is_undefined());
    ///
    /// // Exploded values in certain operations stay defined
    /// let still_defined = exploded / 561;
    /// assert!(!still_defined.is_undefined());
    ///
    /// // But some operations produce undefined results
    /// let undefined_exploded_add = still_defined + 1;
    /// assert!(undefined_exploded_add.is_undefined());
    /// ```
    #[inline]
    pub fn is_undefined(&self) -> bool {
        // Not ambiguous? not undefined!
        if self.exponent != E::AMBIGUOUS_EXPONENT {
            return false;
        }
        // Extract high byte and cast
        let prefix: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();
        if prefix != prefix_i {
            // Prefixes don't match? not undefined!
            return false;
        }
        if prefix == prefix.rotate_right(1) {
            // False for Infinity and Zero
            return false;
        }

        // Check if top 3 bits are equal by pushing 5 bits off
        // ↓↓↓                ↓↓↓
        // □□□xxxxx -5-> □□□□□□□□ - Undefined (℘)
        let top_three = prefix >> 5;
        // Then rotate and compare.  If uniform, they will be equal
        top_three == top_three.rotate_right(1)
    }

    pub fn is_n0(&self) -> bool {
        let prefix: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();
        if prefix != prefix_i {
            // Prefixes don't match? not N0!
            return false;
        }
        prefix == prefix.rotate_right(1)
    }

    pub fn is_n1(&self) -> bool {
        let prefix_r: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();

        // Check for N-1 escaped small patterns by shifting:
        // □■xxxxxx -6-> □□□□□□□■
        // ■□xxxxxx -6-> ■■■■■■■□
        let top_two = prefix_r >> 6;
        if top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8 {
            return true;
        }

        let top_two = prefix_i >> 6;
        top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8
    }

    pub fn is_n2(&self) -> bool {
        let prefix_r: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();

        // Check for N-1 patterns by shifting:
        // □■xxxxxx -6-> □□□□□□□■
        // ■□xxxxxx -6-> ■■■■■■■□
        let top_two = prefix_r >> 6;
        if top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8 {
            return false;
        }

        let top_two = prefix_i >> 6;
        if top_two == 0b00000001u8 as i8 || top_two == 0b11111110u8 as i8 {
            return false;
        }
        // Check for N-2 patterns by shifting:
        // □□■xxxxx -5-> □□□□□□□■
        // ■■□xxxxx -5-> ■■■■■■■□
        let top_three = prefix_r >> 5;
        if top_three == 0b00000001u8 as i8 || top_three == 0b11111110u8 as i8 {
            return true;
        }

        let top_three = prefix_i >> 5;
        top_three == 0b00000001u8 as i8 || top_three == 0b11111110u8 as i8
    }

    /// Returns true if this Circle's magnitude is negligible `[0]`, `[↓]`
    ///
    /// # Description
    ///
    /// Negligible values have effectively zero magnitude.
    /// This includes both actual Zero and vanished values that have become so small they no longer meaningfully contribute to addition or subtraction operations.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[↓]` ➔ `true` Vanished values
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF4E5};
    ///
    /// // Zero: The original negligible number
    /// let zero = Circle::<i16, i32>::ZERO;
    /// assert!(zero.is_negligible());
    ///
    /// // A Circle so small it's effectively zero
    /// let vanished = CircleF4E5::MIN_POS / 57;
    /// assert!(vanished.is_negligible());
    ///
    /// // Normal values are not negligible
    /// let normal = CircleF4E5::from((1729, -104.84));
    /// assert!(!normal.is_negligible());
    ///
    /// // Exploded values are not negligible
    /// let huge = CircleF4E5::MAX_REAL * CircleF4E5::MIN_REAL;
    /// assert!(!huge.is_negligible());
    ///
    /// // Infinity is not negligible
    /// let infinity = CircleF4E5::ONE / 0;
    /// assert!(!infinity.is_negligible());
    ///
    /// // Undefined Circles are not negligible
    /// let undefined = CircleF4E5::ZERO / 0;
    /// assert!(!undefined.is_negligible());
    /// ```
    #[inline]
    pub fn is_negligible(&self) -> bool {
        if self.is_zero() {
            return true;
        }
        self.is_n2()
    }

    /// Returns true if this Circle is an infinitesimal value `[↓]` but not Zero `[0]`
    ///
    /// # Description
    ///
    /// Vanished values are Circles that have become so small their magnitude is effectively zero, but they retain their orientation information. They participate in multiplication and division operations, but are treated as Zero in addition and subtraction.
    ///
    /// # Returns
    ///
    /// - `[↓]` ➔ `true` Vanished values
    /// - `[0]` ➔ `false` Zero
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF4E3};
    ///
    /// // Create a ridiculously small Circle
    /// let vanished = Circle::<i16, i8>::MIN_POS.pow(Scalar::<i16, i8>::MAX);
    /// assert!(vanished.vanished());
    ///
    /// // Actual Zero is not vanished - it's truly Zero
    /// let actual_zero = CircleF4E3::ZERO;
    /// assert!(!actual_zero.vanished());
    ///
    /// // Normal values are not vanished
    /// let normal = CircleF4E3::from((42, -0.6));
    /// assert!(!normal.vanished());
    ///
    /// // Large escaped values aren't vanished - they're exploded!
    /// let ginormous = CircleF4E3::MAX.pow(ScalarF4E3::MAX);
    /// assert!(!ginormous.vanished());
    /// assert!(ginormous.exploded());
    ///
    /// // Division by a vanished value produces an exploded result
    /// let exploded = 1 / vanished;
    /// assert!(exploded.exploded());
    /// assert!(!exploded.vanished());
    /// ```
    #[inline]
    pub fn vanished(&self) -> bool {
        self.is_n2()
    }

    /// Returns true if this Circle is ridiculously large `[↑]` but not infinity `[∞]`
    ///
    /// # Description
    ///
    /// Exploded values are Circles that have grown so large their magnitude can no longer be recorded, but they maintain their orientation information. They participate meaningfully in multiplication and division operations, but addition and subtraction with normal numbers will produce undefined results.
    ///
    /// # Returns
    ///
    /// - `[↑]` ➔ `true` Exploded values
    /// - `[0]` ➔ `false` Zero
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF7E7};
    ///
    /// // Create a really big Circle
    /// let gigantic = Circle::<i128, i128>::MAX.square();
    /// assert!(gigantic.exploded());
    ///
    /// // Actual Zero is not exploded
    /// let actual_zero = CircleF7E7::ZERO;
    /// assert!(!actual_zero.exploded());
    ///
    /// // Normal values are not exploded
    /// let normal = CircleF7E7::from((42, 13.5));
    /// assert!(!normal.exploded());
    ///
    /// // Small escaped values aren't exploded - they're vanished!
    /// let tiny = CircleF7E7::MIN_POS / 12;
    /// assert!(!tiny.exploded());
    /// assert!(tiny.vanished());
    ///
    /// // Infinity is not exploded - it's a distinct state
    /// let infinity = 1 / actual_zero;
    /// assert!(!infinity.exploded());
    /// assert!(infinity.is_infinite());
    ///
    /// // Multiplying or dividing exploded stays exploded
    /// let still_exploded = gigantic * 7;
    /// assert!(still_exploded.exploded());
    /// let also_exploded = gigantic / 7;
    /// assert!(also_exploded.exploded());
    ///
    /// // Division by vanished produces exploded
    /// let also_exploded = CircleF7E7::ONE / tiny;
    /// assert!(also_exploded.exploded());
    /// ```
    #[inline]
    pub fn exploded(&self) -> bool {
        if !self.is_normal() {
            return self.is_n1();
        }
        false
    }

    /// Returns true if this Circle is beyond normal magnitude `[↑]` or `[∞]`
    ///
    /// # Description
    ///
    /// Transfinite values are complex numbers that have grown beyond representable magnitude. This includes both directional exploded values `[↑]` that maintain orientation information, and mathematical infinity `[∞]` which represents a singularity, like division by zero.
    ///
    /// # Returns
    ///
    /// - `[↑]` ➔ `true` Exploded values
    /// - `[∞]` ➔ `true` Infinity
    /// - `[0]` ➔ `false` Zero
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E4};
    ///
    /// // Exploded Circle is transfinite
    /// let exploded = CircleF5E4::MAX_REAL * CircleF5E4::MAX_REAL;
    /// assert!(exploded.is_transfinite());
    /// assert!(exploded.exploded()); // But it's not infinity
    ///
    /// // Division by zero produces true mathematical infinity
    /// let infinity = CircleF5E4::ONE / 0;
    /// assert!(infinity.is_transfinite());
    /// assert!(infinity.is_infinite());
    /// assert!(!infinity.exploded()); // Not the same as exploded
    ///
    /// // Normal complex values are not transfinite
    /// let normal = CircleF5E4::from((42, 13));
    /// assert!(!normal.is_transfinite());
    ///
    /// // Zero is not transfinite
    /// let zero = CircleF5E4::ZERO;
    /// assert!(!zero.is_transfinite());
    ///
    /// // Vanished values are not transfinite (they're the opposite!)
    /// let tiny = CircleF5E4::MIN_POS / 1234;
    /// assert!(!tiny.is_transfinite());
    ///
    /// // The reciprocal of transfinite is negligible
    /// let reciprocal = CircleF5E4::ONE / exploded;
    /// assert!(!reciprocal.is_transfinite());
    /// assert!(reciprocal.is_negligible());
    ///
    /// // Math operations with infinity follow mathematical rules
    /// let also_infinity = infinity * CircleF5E4::PI;
    /// assert!(also_infinity.is_transfinite());
    /// assert!(also_infinity.is_infinite());
    ///
    /// // Comparisons with infinity are undefined
    /// let comparison = infinity.max(CircleF5E4::from((1000, 500)));
    /// assert!(comparison.is_undefined());
    /// ```
    #[inline]
    pub fn is_transfinite(&self) -> bool {
        if !self.is_normal() {
            return self.is_n1() || (self.real == F::NEG_ONE && self.imaginary == F::NEG_ONE);
        }
        false
    }

    /// Returns true if this Circle represents a finite complex number `[0]`, `[#,#]`
    ///
    /// # Description
    ///
    /// Finite Circles have known magnitudes and orientations and can participate fully in all arithmetic operations. This includes all normal complex numbers and Zero, but excludes escaped values (exploded and vanished) and undefined states.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[#,#]` ➔ `true` Normal complex numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E4};
    ///
    /// // Normal values are finite
    /// let normal = Circle::<i32, i16>::from((42, 2.1));
    /// assert!(normal.is_finite());
    ///
    /// // Zero is finite
    /// let zero = CircleF5E4::ZERO;
    /// assert!(zero.is_finite());
    ///
    /// // Complex number with i is finite
    /// let complex = CircleF5E4::I;
    /// assert!(complex.is_finite());
    ///
    /// // Exploded values are not finite
    /// let huge = CircleF5E4::MAX_REAL * 2;
    /// assert!(!huge.is_finite());
    ///
    /// // Vanished values are not finite
    /// let tiny = CircleF5E4::MIN_POS / 28;
    /// assert!(!tiny.is_finite());
    ///
    /// // Infinity is not finite
    /// let infinity = CircleF5E4::ONE / 0;
    /// assert!(!infinity.is_finite());
    ///
    /// // Undefined Circles are not finite
    /// let undefined = CircleF5E4::ZERO / 0;
    /// assert!(!undefined.is_finite());
    ///
    /// // Operations that produce normal results usually return finite values
    /// let still_finite = normal + CircleF5E4::ONE;
    /// assert!(still_finite.is_finite());
    ///
    /// // Operations that exceed representable range escape finiteness
    /// let no_longer_finite = CircleF5E4::MAX_REAL + CircleF5E4::MAX_REAL / 2;
    /// assert!(!no_longer_finite.is_finite());
    /// ```
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.is_normal() || self.is_zero()
    }

    /// Returns true if this Circle is the true origin point `[0]`
    ///
    /// # Description
    ///
    /// Zero represents the absence of magnitude in Spirix. Unlike vanished values which approach but never equal Zero, this is the genuine, mathematical Zero that serves as the additive identity.
    ///
    /// # Returns
    ///
    /// - `[0]` ➔ `true` Zero
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[∞]` ➔ `false` Infinity
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF6E4};
    ///
    /// // The one and only Zero
    /// let zero = Circle::<i64, i16>::ZERO;
    /// assert!(zero.is_zero());
    ///
    /// // Even a very small Circle is not Zero
    /// let tiny = CircleF6E4::MIN_POS / 61;
    /// assert!(!tiny.is_zero());
    /// assert!(tiny.vanished());  // It's vanished, not Zero
    ///
    /// // Normal values are not Zero
    /// let normal = CircleF6E4::from((42, 17));
    /// assert!(!normal.is_zero());
    ///
    /// // Exploded values are not Zero
    /// let huge = CircleF6E4::MAX_REAL * CircleF6E4::MAX_REAL;
    /// assert!(!huge.is_zero());
    ///
    /// // Infinity is not Zero
    /// let infinity = CircleF6E4::ONE / 0;
    /// assert!(!infinity.is_zero());
    ///
    /// // Undefined states aren't Zero
    /// let undefined = CircleF6E4::ZERO / 0;
    /// assert!(!undefined.is_zero());
    ///
    /// // Mathematical operations with Zero behave as expected
    /// let still_zero = zero * CircleF6E4::from((3.5, 4));
    /// assert!(still_zero.is_zero());
    ///
    /// let normal_again = still_zero + CircleF6E4::from((163, -16));
    /// assert!(!normal_again.is_zero());
    /// assert!(normal_again.is_normal());
    /// ```
    #[inline]
    pub fn is_zero(&self) -> bool {
        let prefix_r: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();
        prefix_r == 0 && prefix_i == 0
    }

    /// Returns true if this Circle represents mathematical infinity `[∞]`
    ///
    /// # Description
    ///
    /// Infinity represents the result of operations like division by zero or other mathematical concepts that produce a singularity. In Spirix, this corresponds to the "point at infinity" on the Riemann sphere, a single entity that represents the limit of all approaches to infinity.
    ///
    /// # Returns
    ///
    /// - `[∞]` ➔ `true` Infinity
    /// - `[0]` ➔ `false` Zero
    /// - `[#,#]` ➔ `false` Normal complex numbers
    /// - `[↑]` ➔ `false` Exploded values
    /// - `[↓]` ➔ `false` Vanished values
    /// - `[℘?]` ➔ `false` Undefined states
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E4};
    ///
    /// // Division by zero produces infinity
    /// let infinity = CircleF5E4::ONE / 0;
    /// assert!(infinity.is_infinite());
    ///
    /// // Exploded values are not infinity
    /// let exploded = CircleF5E4::MAX_REAL * CircleF5E4::MAX_REAL;
    /// assert!(!exploded.is_infinite());
    /// assert!(exploded.exploded());
    ///
    /// // Normal values are not infinity
    /// let normal = CircleF5E4::from((42, 7.1));
    /// assert!(!normal.is_infinite());
    ///
    /// // Zero is not infinity
    /// let zero = CircleF5E4::ZERO;
    /// assert!(!zero.is_infinite());
    ///
    /// // Undefined values are not infinity
    /// let undefined = CircleF5E4::ZERO / 0;
    /// assert!(!undefined.is_infinite());
    ///
    /// // Operations with infinity follow mathematical rules
    /// let still_infinity = infinity * CircleF5E4::from((3, -4.5));
    /// assert!(still_infinity.is_infinite());
    ///
    /// // Indeterminate forms with infinity produce undefined results
    /// let undefined = infinity + infinity;
    /// assert!(undefined.is_undefined());
    /// ```
    #[inline]
    pub fn is_infinite(&self) -> bool {
        let prefix_r: i8 = self.real.sa();
        let prefix_i: i8 = self.imaginary.sa();
        prefix_r == -1 && prefix_i == -1
    }

    /// Negates this Circle in place
    ///
    /// # Description
    ///
    /// Computes the negation by inverting both the real and imaginary components.
    /// For a complex number a + b*i, the negation is -a - b*i.
    ///
    /// For escaped values, this preserves the escape state while inverting the orientation. Undefined states are preserved.
    ///
    /// This is a low-level internal method. Users should use the `-` operator instead.
    ///
    /// # Effects
    ///
    /// - For normal values: Their negation with the same magnitude
    /// - For escaped values: The negation with the same escape state
    /// - For Zero: Zero (unchanged)
    /// - For Infinity: Infinity (unchanged)
    /// - For undefined states: The same undefined state
    pub(crate) fn circle_negate(&mut self) {
        if self.exponent != E::AMBIGUOUS_EXPONENT {
            let one: E = 1u8.as_();
            if self.real == F::NEG_ONE_FRACTION {
                self.real = F::POS_ONE_FRACTION;
                self.imaginary = (self.imaginary >> 1isize).wrapping_neg();
                self.exponent = self.exponent.wrapping_add(&one);
                return;
            } else if self.imaginary == F::NEG_ONE_FRACTION {
                self.imaginary = F::POS_ONE_FRACTION;
                self.real = (self.real >> 1isize).wrapping_neg();
                self.exponent = self.exponent.wrapping_add(&one);
                return;
            }
            self.real = self.real.wrapping_neg();
            self.imaginary = self.imaginary.wrapping_neg();
            self.normalize();
        } else {
            // Extract high byte and cast
            let prefix: i8 = self.real.sa();
            let prefix_i: i8 = self.imaginary.sa();
            if prefix == prefix_i {
                // Test for undefined, Zero and Infinity
                // Check if top 3 bits are equal by pushing 5 bits off
                // ↓↓↓                ↓↓↓
                // □□□xxxxx -5-> □□□□□□□□ - Undefined (℘)
                let top_three = prefix >> 5;
                // Then rotate and compare.  If uniform, they will be equal
                // True for undefined, Zero and Infinity
                if top_three == top_three.rotate_right(1) {
                    return;
                }
            }
            if self.vanished() {
                self.real = self.real.wrapping_neg();
                self.imaginary = self.imaginary.wrapping_neg();
                self.normalize_vanished();
            } else {
                if self.real == F::NEG_ONE_FRACTION {
                    self.real = F::POS_ONE_FRACTION;
                    self.imaginary = (self.imaginary >> 1isize).wrapping_neg();
                } else if self.imaginary == F::NEG_ONE_FRACTION {
                    self.imaginary = F::POS_ONE_FRACTION;
                    self.real = (self.real >> 1isize).wrapping_neg();
                } else {
                    self.real = self.real.wrapping_neg();
                    self.imaginary = self.imaginary.wrapping_neg();
                    self.normalize_exploded()
                }
            }
        }
    }

    /// Returns the complex conjugate of this Circle
    ///
    /// # Description
    ///
    /// Computes the complex conjugate by negating the imaginary component, while preserving the real component. For a complex number a + b*i, the conjugate is a - b*i.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[#,#]` Complex conjugate
    /// - `[↑]` ➔ `[↑]` Conjugate with exploded state
    /// - `[↓]` ➔ `[↓]` Conjugate with vanished state
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` Same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Conjugate of a standard complex number
    /// let complex = Circle::<i32, i8>::from((3, 5.6));
    /// let conj = complex.conjugate();
    /// assert!(conj.r() == 3);
    /// assert!(conj.i() == -5.6);
    /// assert!(complex.magnitude() == conj.magnitude());
    ///
    /// // Conjugate of a pure real number doesn't change it
    /// let real = CircleF5E3::from((42, 0));
    /// assert!(real.conjugate() == real);
    ///
    /// // Conjugate of a pure imaginary number negates it
    /// let imag = CircleF5E3::from((0, 7));
    /// assert!(imag.conjugate().i() == -7);
    ///
    /// // Conjugate of Zero is Zero
    /// let zero = CircleF5E3::ZERO;
    /// assert!(zero.conjugate() == zero);
    ///
    /// // Conjugate of Infinity is Infinity
    /// let infinity = CircleF5E3::ONE / 0;
    /// assert!(infinity.conjugate().is_infinite());
    ///
    /// // Conjugate of vanished value preserves vanished state
    /// let tiny = CircleF5E3::MIN_POS / 42;
    /// assert!(tiny.conjugate().vanished());
    ///
    /// // Conjugate of undefined remains undefined
    /// let undefined = CircleF5E3::ZERO / 0;
    /// assert!(undefined.conjugate().is_undefined());
    /// ```
    pub fn conjugate(&self) -> Circle<F, E> {
        if self.is_normal() {
            if self.imaginary == F::NEG_ONE_FRACTION {
                return Circle {
                    real: self.real >> 1isize,
                    imaginary: F::POS_ONE_FRACTION,
                    exponent: self.exponent.wrapping_add(&E::ONE),
                };
            }
            let mut conjugate = Circle {
                real: self.real,
                imaginary: self.imaginary.wrapping_neg(),
                exponent: self.exponent,
            };
            conjugate.normalize();
            conjugate
        } else if self.vanished() {
            let mut conjugate = Circle {
                real: self.real,
                imaginary: self.imaginary.wrapping_neg(),
                exponent: self.exponent,
            };
            conjugate.normalize_vanished();
            conjugate
        } else if self.exploded() {
            if self.imaginary == F::NEG_ONE_FRACTION {
                return Circle {
                    real: (self.real >> 1isize).wrapping_neg(),
                    imaginary: F::POS_ONE_FRACTION,
                    exponent: self.exponent,
                };
            } else {
                let mut conjugate = Circle {
                    real: self.real,
                    imaginary: self.imaginary.wrapping_neg(),
                    exponent: self.exponent,
                };
                conjugate.normalize_exploded();
                conjugate
            }
        } else {
            *self
        }
    }

    /// Computes the complex conjugate of this Circle in place
    ///
    /// # Description
    ///
    /// Performs an in-place computation of the complex conjugate by negating the imaginary component, while preserving the real component. For a complex number a + b*i, the conjugate is a - b*i.
    ///
    /// # Effects
    ///
    /// - `[#,#]` ➔ `[#,#]` Negates imaginary part, preserves real
    /// - `[↑]` ➔ `[↑]` Preserves escape state while flipping orientation
    /// - `[↓]` ➔ `[↓]` Preserves escape state while flipping orientation
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` Same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Conjugate of a standard complex number
    /// let mut complex = Circle::<i32, i8>::from((3, 5.6));
    /// let magnitude = complex.magnitude();
    /// complex.conjugate_mut();
    /// assert!(complex.r() == 3);
    /// assert!(complex.i() == -5.6);
    /// assert!(complex.magnitude() == magnitude);
    ///
    /// // Conjugate of a pure real number doesn't change it
    /// let mut real = CircleF5E3::from((42, 0));
    /// let original = real;
    /// real.conjugate_mut();
    /// assert!(real == original);
    ///
    /// // Conjugate of a pure imaginary number negates it
    /// let mut imag = CircleF5E3::from((0, 7));
    /// imag.conjugate_mut();
    /// assert!(imag.i() == -7);
    ///
    /// // Conjugate of Zero is Zero
    /// let mut zero = CircleF5E3::ZERO;
    /// zero.conjugate_mut();
    /// assert!(zero.is_zero());
    ///
    /// // Conjugate of Infinity remains Infinity
    /// let mut infinity = CircleF5E3::ONE / 0;
    /// infinity.conjugate_mut();
    /// assert!(infinity.is_infinite());
    ///
    /// // Conjugate of vanished value preserves vanished state
    /// let mut tiny = CircleF5E3::MIN_POS / 42;
    /// tiny.conjugate_mut();
    /// assert!(tiny.vanished());
    ///
    /// // Conjugate of undefined remains undefined
    /// let mut undefined = CircleF5E3::ZERO / 0;
    /// undefined.conjugate_mut();
    /// assert!(undefined.is_undefined());
    /// ```
    pub fn conjugate_mut(&mut self) {
        if self.is_normal() {
            if self.imaginary == F::NEG_ONE_FRACTION {
                self.real = self.real >> 1isize;
                self.imaginary = F::POS_ONE_FRACTION;
                self.exponent = self.exponent.wrapping_add(&E::ONE);
                return;
            }
            self.imaginary = self.imaginary.wrapping_neg();
            self.normalize()
        } else if self.vanished() {
            self.imaginary = self.imaginary.wrapping_neg();
            self.normalize_vanished();
        } else if self.exploded() {
            if self.imaginary == F::NEG_ONE_FRACTION {
                self.imaginary = F::POS_ONE_FRACTION;
                self.real = (self.real >> 1isize).wrapping_neg();
            } else {
                self.imaginary = self.imaginary.wrapping_neg();
                self.normalize_exploded()
            }
        }
    }

    /// Returns the magnitude of this Circle
    ///
    /// # Description
    ///
    /// Computes the magnitude of this Circle as a Scalar.
    /// For a complex number a + b*i, the magnitude is √(a² + b²).
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[+#]` Scalar representing distance from origin
    /// - `[↑]` ➔ `[↑]` Scalar with exploded state
    /// - `[↓]` ➔ `[↓]` Scalar with vanished state
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` Scalar with same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF4E4, ScalarF4E4};
    ///
    /// // Magnitude of a standard 3-4-5 triangle
    /// let complex = Circle::<i16, i16>::from((3, 4));
    /// assert!(complex.magnitude() == 5);
    ///
    /// // Magnitude of a pure real number is its absolute value
    /// let real = CircleF4E4::from((-42, 0));
    /// assert!(real.magnitude() == 42);
    ///
    /// // Magnitude of a pure imaginary number is its absolute value
    /// let imag = CircleF4E4::from((0, -7.2));
    /// assert!(imag.magnitude() == 7.2);
    ///
    /// // Magnitude of Zero is Zero
    /// let zero = CircleF4E4::ZERO;
    /// assert!(zero.magnitude() == ScalarF4E4::ZERO);
    ///
    /// // Magnitude of Infinity is Infinity
    /// let infinity = CircleF4E4::ONE / 0;
    /// assert!(infinity.magnitude().is_infinite());
    ///
    /// // Magnitude of vanished value is a vanished Scalar
    /// let tiny = CircleF4E4::MIN_POS / 100;
    /// assert!(tiny.magnitude().vanished());
    ///
    /// // Magnitude of undefined is undefined
    /// let undefined = CircleF4E4::ZERO / 0;
    /// assert!(undefined.magnitude().is_undefined());
    /// ```
    pub fn magnitude(&self) -> Scalar<F, E> {
        let magnitude_squared = self.magnitude_squared();
        magnitude_squared.sqrt()
    }

    /// Returns the square of the magnitude of this Circle
    ///
    /// # Description
    ///
    /// Computes the squared magnitude of this Circle as a Scalar, avoiding the square root operation. For a complex number a + b*i, the squared magnitude is a * a + b * b.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[+#]` Scalar representing squared distance from origin
    /// - `[↑]` ➔ `[↑]` Scalar with exploded state
    /// - `[↓]` ➔ `[↓]` Scalar with vanished state
    /// - `[0]` ➔ `[0]` Zero unchanged
    /// - `[∞]` ➔ `[∞]` Infinity unchanged
    /// - `[℘?]` ➔ `[℘?]` Scalar with same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF6E2, ScalarF6E2};
    ///
    /// // Squared magnitude of a standard 3-4-5 triangle
    /// let complex = Circle::<i64, i8>::from((3, 4));
    /// assert!(complex.magnitude_squared() == 25);
    ///
    /// // Squared magnitude of a pure real number
    /// let real = CircleF6E2::from((4, 0));
    /// assert!(real.magnitude_squared() == 16);
    ///
    /// // Squared magnitude of a pure imaginary number
    /// let imag = CircleF6E2::from((0, 5));
    /// assert!(imag.magnitude_squared() == 25);
    ///
    /// // Squared magnitude of Zero is Zero
    /// let zero = CircleF6E2::ZERO;
    /// assert!(zero.magnitude_squared() == ScalarF6E2::ZERO);
    ///
    /// // Squared magnitude of Infinity is Infinity
    /// let infinity = CircleF6E2::ONE / 0;
    /// assert!(infinity.magnitude_squared().is_infinite());
    ///
    /// // Squared magnitude of undefined is undefined
    /// let undefined = CircleF6E2::ZERO / 0;
    /// assert!(undefined.magnitude_squared().is_undefined());
    /// ```
    pub fn magnitude_squared(&self) -> Scalar<F, E> {
        let real_squared = self.r().square();
        let imaginary_squared = self.i().square();
        real_squared + imaginary_squared
    }

    /// Returns the reciprocal (multiplicative inverse) of this Circle
    ///
    /// # Description
    ///
    /// Computes the reciprocal 1/z of this Circle. For a complex number a + b*i,
    /// the reciprocal is (a - b*i)/(a² + b²), which equals conjugate(z) / |z|².
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[#,#]` Reciprocal with same precision
    /// - `[↑]` ➔ `[↓]` Large values become small (exploded → vanished)
    /// - `[↓]` ➔ `[↑]` Small values become large (vanished → exploded)
    /// - `[0]` ➔ `[∞]` Zero becomes Infinity
    /// - `[∞]` ➔ `[0]` Infinity becomes Zero
    /// - `[℘?]` ➔ `[℘?]` Same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Reciprocal of a real number
    /// let real = Circle::<i32, i8>::from(4);
    /// let recip = real.reciprocal();
    /// assert!(recip.r() == 0.25);
    /// assert!(recip.i() == 0);
    ///
    /// // Reciprocal of a pure imaginary number
    /// let imag = CircleF5E3::from((0, 2));
    /// let recip_imag = imag.reciprocal();
    /// assert!(recip_imag.r() == 0);
    /// assert!(recip_imag.i() == -0.5);
    ///
    /// // Reciprocal of a general complex number
    /// let complex = CircleF5E3::from((3, 4));
    /// let recip_complex = complex.reciprocal();
    /// assert!((recip_complex.r() - 0.12).magnitude() < 0.01);
    /// assert!((recip_complex.i() + 0.16).magnitude() < 0.01);
    ///
    /// // Reciprocal relationships
    /// let z = CircleF5E3::from((1, 1));
    /// let recip_z = z.reciprocal();
    /// assert!((z * recip_z - CircleF5E3::ONE).magnitude() < 0.01);
    ///
    /// // Reciprocal of Zero is Infinity
    /// let zero = CircleF5E3::ZERO;
    /// assert!(zero.reciprocal().is_infinite());
    ///
    /// // Reciprocal of Infinity is Zero
    /// let infinity = CircleF5E3::ONE / 0;
    /// assert!(infinity.reciprocal().is_zero());
    ///
    /// // Reciprocal of undefined remains undefined
    /// let undefined = CircleF5E3::ZERO / 0;
    /// assert!(undefined.reciprocal().is_undefined());
    /// ```
    pub fn reciprocal(&self) -> Circle<F, E> {
        1 / self
    }

    /// Returns the normalized unit vector form of this Circle
    ///
    /// # Description
    ///
    /// Creates a unit Circle (magnitude 1) pointing in the same direction as this Circle. This preserves the orientation while normalizing the magnitude, by dividing the Circle by its magnitude.
    ///
    /// # Returns
    ///
    /// - `[#,#]` ➔ `[#,#]` Unit Circle with same orientation
    /// - `[↑]` ➔ `[#,#]` Normal unit Circle with same orientation (escape state removed)
    /// - `[↓]` ➔ `[#,#]` Normal unit Circle with same orientation (escape state removed)
    /// - `[0]` ➔ `[℘±∅]` Undefined value with SIGN_INDETERMINATE pattern
    /// - `[∞]` ➔ `[℘±∅]` Undefined value with SIGN_INDETERMINATE pattern
    /// - `[℘?]` ➔ `[℘?]` Same undefined state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E5};
    ///
    /// // Normalizing a standard complex number
    /// let complex = Circle::<i32, i32>::from((-3, 0));
    /// let unit = complex.sign();
    /// assert!(unit.r() == -1);
    /// assert!(unit.i() == 0);
    ///
    /// // Normalizing a complex number with both components
    /// let z = CircleF5E5::from((1, 1));
    /// let unit_z = z.sign();
    /// assert!(unit_z.magnitude() == 1);
    ///
    /// // Normalizing a pure real number
    /// let real = CircleF5E5::from((42, 0));
    /// assert!(real.sign().r() == 1);
    /// assert!(real.sign().i() == 0);
    ///
    /// // Normalizing a pure imaginary number
    /// let imag = CircleF5E5::from((0, 7));
    /// assert!(imag.sign().r() == 0);
    /// assert!(imag.sign().i() == 1);
    ///
    /// // Normalizing an exploded value removes the escape state
    /// let huge = CircleF5E5::MAX_REAL * 2;
    /// assert!(huge.exploded());
    /// assert!(!huge.sign().exploded());
    ///
    /// // Zero has no orientation to normalize
    /// let zero = CircleF5E5::ZERO;
    /// assert!(zero.sign().is_undefined());
    ///
    /// // Infinity's direction is undefined
    /// let infinity = CircleF5E5::ONE / 0;
    /// assert!(infinity.sign().is_undefined());
    /// ```
    pub fn sign(&self) -> Circle<F, E> {
        if self.exponent == E::AMBIGUOUS_EXPONENT {
            let prefix_r: i8 = self.real.sa();
            let prefix_i: i8 = self.imaginary.sa();
            if prefix_r == prefix_i {
                if prefix_r == prefix_r.rotate_right(1) {
                    // Zero and Infinity check
                    let prefix: F = SIGN_INDETERMINATE.prefix.sa();
                    return Self {
                        real: prefix,
                        imaginary: prefix,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                let top_three = prefix_r >> 5;
                // Undefined check
                if top_three == top_three.rotate_right(1) {
                    return *self;
                }
            }
        }
        let result = Circle {
            real: self.real,
            imaginary: self.imaginary,
            exponent: E::ZERO,
        };
        return result / result.magnitude();
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
    pub(crate) fn normalize(&mut self) {
        let shift_r = self.real.leading_ones().max(self.real.leading_zeros());
        let shift_i = self
            .imaginary
            .leading_ones()
            .max(self.imaginary.leading_zeros());
        let shift = shift_r.min(shift_i);

        if shift > 1 {
            let shift = shift as isize;
            let new_exponent;
            if shift == F::FRACTION_BITS {
                if !self.real.is_negative() && !self.imaginary.is_negative() {
                    self.exponent = E::AMBIGUOUS_EXPONENT;
                    return;
                }
                let s: E = shift.wrapping_add(1).as_();
                new_exponent = self.exponent.wrapping_sub(&s);
            } else {
                let s: E = shift.as_();
                new_exponent = self.exponent.wrapping_sub(&s);
            }

            if self.exponent.is_negative() && !new_exponent.is_negative() {
                self.exponent = E::AMBIGUOUS_EXPONENT;
                self.real = self.real << shift.wrapping_sub(2);
                self.imaginary = self.imaginary << shift.wrapping_sub(2);
            } else {
                self.exponent = new_exponent.wrapping_add(&E::ONE);
                self.real = self.real << shift.wrapping_sub(1);
                self.imaginary = self.imaginary << shift.wrapping_sub(1);
            }
        }
    }

    /// Normalizes a vanished Scalar by shifting its fraction to the N-2 position.  
    /// Sign bits occupy N-0 and N-1, exponent is not touched
    ///  
    /// Example bit positions:
    /// 01234567...  
    /// □□■xxxxx... - Vanished positive numbers  
    /// ■■□xxxxx... - Vanished negative numbers  
    pub(crate) fn normalize_vanished(&mut self) {
        let shift_r = self.real.leading_ones().max(self.real.leading_zeros());
        let shift_i = self
            .imaginary
            .leading_ones()
            .max(self.imaginary.leading_zeros());
        let shift: isize = shift_r.min(shift_i).as_();

        if shift > 2 {
            self.real = self.real << shift.wrapping_sub(2);
            self.imaginary = self.imaginary << shift.wrapping_sub(2);
        } else if shift < 2 {
            self.real = self.real >> 2isize.wrapping_sub(shift);
            self.imaginary = self.imaginary >> 2isize.wrapping_sub(shift);
        }
    }

    /// Normalizes an exploded Scalar by shifting the fraction left until the most significant bit is in the N-1 position, exponent is not touched.  
    ///  
    /// Example bit positions:
    /// 01234567...
    /// □■xxxxxx... - Exploded positive numbers  
    /// ■□xxxxxx... - Exploded negative numbers  
    pub(crate) fn normalize_exploded(&mut self) {
        let shift_r = self.real.leading_ones().max(self.real.leading_zeros());
        let shift_i = self
            .imaginary
            .leading_ones()
            .max(self.imaginary.leading_zeros());
        let shift = shift_r.min(shift_i);

        if shift > 1 {
            let shift = shift as isize;
            self.real = self.real << shift.wrapping_sub(1);
            self.imaginary = self.imaginary << shift.wrapping_sub(1);
        }
    }
}
