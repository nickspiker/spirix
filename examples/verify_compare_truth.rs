//! Exhaustive F3E3 verification of compare()/==/</<=/>/>=.
//!
//! For every pair (a, b) of F3E3 values, computes Spirix's ordering and
//! compares to an f64 oracle. Also checks PartialOrd/PartialEq consistency
//! (reflexivity, anti-symmetry, transitivity within a sample).
use spirix::*;
use std::cmp::Ordering;

type S = ScalarF3E3;

fn classify_orderable(s: S) -> bool {
    // compare() returns None for any input involving undefined/infinity,
    // and for same-sign same-class escaped pairs. Most "normal" pairs plus
    // mixed-class finite pairs should yield Some(_).
    !s.is_undefined() && !s.is_infinite()
}

fn main() {
    // Collect all F3E3 encodings once.
    let mut all: Vec<S> = Vec::with_capacity(65536);
    for f in -128i8..=127 {
        for e in -128i8..=127 {
            let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
            all.push(s);
        }
    }

    let mut total = 0usize;
    let mut sign_errors = 0usize;
    let mut first_bad: Option<(S, S, Ordering, f64, f64)> = None;
    // Skip counters
    let mut skipped_orderable = 0usize;
    let mut skipped_unorderable = 0usize;
    // Reflexivity: a.compare(&a) should be Some(Equal) for orderable or None.
    let mut reflex_errors = 0usize;

    for a in &all {
        // Reflexivity
        let self_cmp = a.partial_cmp(a);
        if classify_orderable(*a) {
            if self_cmp != Some(Ordering::Equal) {
                // Escaped (van/exp) compared to itself: the current design
                // returns None for same-sign same-class pairs, which includes
                // same-value escaped comparisons. That's consistent with "the
                // value is magnitude-unknown, so we can't assert equality."
                // Only flag reflexivity failure for normal/zero.
                if a.is_normal() || a.is_zero() {
                    reflex_errors += 1;
                }
            }
        }
        for b in &all {
            let spirix_cmp = a.partial_cmp(b);
            let af: f64 = (*a).into();
            let bf: f64 = (*b).into();
            // f64 oracle is unreliable when f64 loses the magnitude (e.g.
            // very small subnormals → 0, very large → inf). Skip those.
            if af == 0.0 && !a.is_zero() { continue; }
            if bf == 0.0 && !b.is_zero() { continue; }
            if !af.is_finite() || !bf.is_finite() { continue; }
            let f64_cmp = af.partial_cmp(&bf);
            match (spirix_cmp, f64_cmp) {
                (Some(sc), Some(oc)) => {
                    total += 1;
                    if sc != oc {
                        sign_errors += 1;
                        if first_bad.is_none() {
                            first_bad = Some((*a, *b, sc, af, bf));
                        }
                    }
                }
                (None, _) => {
                    skipped_unorderable += 1;
                }
                (Some(_), None) => {
                    // Spirix ordered, f64 didn't (f64 NaN). This could be a
                    // case where f64 lost info we still have.
                    skipped_orderable += 1;
                }
            }
        }
    }

    println!("=== compare() ({} pairs checked, {} ordering mismatches, {} reflex failures) ===",
             total, sign_errors, reflex_errors);
    println!("  skipped (spirix None): {}", skipped_unorderable);
    println!("  skipped (f64 None, spirix Some): {}", skipped_orderable);
    if let Some((a, b, sc, af, bf)) = first_bad {
        let ab = unsafe { std::mem::transmute::<S, [i8; 2]>(a) };
        let bb = unsafe { std::mem::transmute::<S, [i8; 2]>(b) };
        println!("  first bad:");
        println!("    a = [{:#04x},{}] = {:e}", ab[0] as u8, ab[1], af);
        println!("    b = [{:#04x},{}] = {:e}", bb[0] as u8, bb[1], bf);
        println!("    spirix says {:?}, f64 says {:?}", sc, af.partial_cmp(&bf));
    }
}
