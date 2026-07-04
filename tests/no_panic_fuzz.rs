//! Production safety net: every op, over every representable bit pattern (or a large deterministic sample), must (a) never panic and (b) return a value that classifies into EXACTLY one class.
//! Bit patterns here include non-canonical ones a user could construct via the pub fields — ops must stay total over the whole representation space, not just values arithmetic can produce.
//! F3E3 unary coverage is exhaustive (all 65,536 patterns); binary pairs and the wide F7E7 width use a fixed-seed LCG so runs are deterministic.

use spirix::*;

type S = ScalarF3E3;
type C = CircleF3E3;
type SW = ScalarF7E7;

/// Exactly one Scalar class predicate must hold for ANY bit pattern.
fn assert_one_class_s(v: &S, what: &str) {
    let n = [v.is_zero(), v.is_infinite(), v.vanished(), v.exploded(), v.is_undefined(), v.is_normal()]
        .iter()
        .filter(|&&b| b)
        .count();
    assert!(n == 1, "{what}: {n} classes claim {:?} (frac {:#010b}, exp {:#010b})", v, v.fraction as u8, v.exponent as u8);
}

fn assert_one_class_c(v: &C, what: &str) {
    let n = [v.is_zero(), v.is_infinite(), v.vanished(), v.exploded(), v.is_undefined(), v.is_normal()]
        .iter()
        .filter(|&&b| b)
        .count();
    assert!(n == 1, "{what}: {n} classes claim {:?}", v);
}

/// Deterministic 64-bit LCG (Knuth MMIX constants) — fixed seed, reproducible failures.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
}

fn s_from(bits: u16) -> S {
    Scalar::<i8, i8> {
        fraction: (bits >> 8) as i8,
        exponent: bits as i8,
    }
}

#[test]
fn scalar_unary_total_over_all_patterns() {
    // All 65,536 (fraction, exponent) patterns thru every unary op.
    for bits in 0u16..=u16::MAX {
        let v = s_from(bits);
        assert_one_class_s(&v, "input itself");
        let results = [
            ("neg", -v),
            ("not", !v),
            ("abs", v.magnitude()),
            ("sign", v.sign()),
            ("recip", v.reciprocal()),
            ("square", v.square()),
            ("sqrt", v.sqrt()),
            ("ln", v.ln()),
            ("lb", v.lb()),
            ("exp", v.exp()),
            ("powb", v.powb()),
            ("floor", v.floor()),
            ("ceil", v.ceil()),
            ("round", v.round()),
            ("frac", v.frac()),
            ("sin", v.sin()),
            ("cos", v.cos()),
            ("tan", v.tan()),
            ("asin", v.asin()),
            ("acos", v.acos()),
            ("atan", v.atan()),
            ("sinh", v.sinh()),
            ("cosh", v.cosh()),
            ("tanh", v.tanh()),
        ];
        for (name, r) in results {
            assert_one_class_s(&r, name);
        }
    }
}

#[test]
fn scalar_binary_total_over_sampled_pairs() {
    let mut rng = Lcg(0x5312_9E1F_00D5_EED5);
    for _ in 0..200_000 {
        let w = rng.next();
        let a = s_from(w as u16);
        let b = s_from((w >> 16) as u16);
        let sh = (w >> 32) as i32 % 300; // deliberately past the E-width saturation point
        let results = [
            ("+", a + b),
            ("-", a - b),
            ("*", a * b),
            ("/", a / b),
            ("%", a % b),
            ("pow", a.pow(b)),
            ("log", a.log(b)),
            ("&", a & b),
            ("|", a | b),
            ("^", a ^ b),
            ("<<", a << sh),
            (">>", a >> sh),
            ("min", a.min(b)),
            ("max", a.max(b)),
            ("atan2", a.atan2(b)),
            ("clamp", a.clamp(b.min(b), b.max(b))),
        ];
        for (name, r) in results {
            assert_one_class_s(&r, name);
        }
        // Comparisons must be total functions too (bool out, no panic).
        let _ = (a < b, a > b, a == b, a <= b, a >= b);
    }
}

#[test]
fn wide_width_total_over_sampled_patterns() {
    // F7E7 (i128 fraction + i128 exponent) exercises the widest arithmetic paths (I256 wide ops).
    let mut rng = Lcg(0xFEED_FACE_CAFE_BEEF);
    let mut wide = |r: &mut Lcg| -> i128 { ((r.next() as i128) << 64) | r.next() as i128 };
    for _ in 0..2_000 {
        let a = Scalar::<i128, i128> { fraction: wide(&mut rng), exponent: wide(&mut rng) };
        let b = Scalar::<i128, i128> { fraction: wide(&mut rng), exponent: wide(&mut rng) };
        let n = [a.is_zero(), a.is_infinite(), a.vanished(), a.exploded(), a.is_undefined(), a.is_normal()].iter().filter(|&&x| x).count();
        assert!(n == 1, "F7E7 pattern claims {n} classes");
        // The heavy ops: multiply/divide/add drive the I256 machinery; sqrt/exp the iterative loops.
        let results: [SW; 7] = [a + b, a - b, a * b, a / b, a % b, a.sqrt(), a.magnitude()];
        for r in results {
            let n = [r.is_zero(), r.is_infinite(), r.vanished(), r.exploded(), r.is_undefined(), r.is_normal()].iter().filter(|&&x| x).count();
            assert!(n == 1, "F7E7 result claims {n} classes");
        }
    }
}

#[test]
fn circle_total_over_sampled_patterns() {
    let mut rng = Lcg(0xC19C_1E5E_ED00_0001);
    for _ in 0..100_000 {
        let w = rng.next();
        let a = Circle::<i8, i8> { real: w as i8, imaginary: (w >> 8) as i8, exponent: (w >> 16) as i8 };
        let b = Circle::<i8, i8> { real: (w >> 24) as i8, imaginary: (w >> 32) as i8, exponent: (w >> 40) as i8 };
        assert_one_class_c(&a, "circle input");
        let results = [
            ("neg", -a),
            ("conj", a.conjugate()),
            ("recip", a.reciprocal()),
            ("square", a.square()),
            ("sqrt", a.sqrt()),
            ("ln", a.ln()),
            ("exp", a.exp()),
            ("+", a + b),
            ("-", a - b),
            ("*", a * b),
            ("/", a / b),
            ("z^w", a.pow(b)),
        ];
        for (name, r) in results {
            assert_one_class_c(&r, name);
        }
        // Circle^scalar and magnitude (Scalar out).
        let p = Scalar::<i8, i8> { fraction: (w >> 48) as i8, exponent: (w >> 56) as i8 };
        assert_one_class_c(&a.pow(p), "z^s");
        assert_one_class_s(&a.magnitude(), "|z|");
    }
}
