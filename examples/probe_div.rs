//! Hunt for the remaining div truth-table failures. Scan ALL F3E3 pairs and
//! print the first few where class is expected Exploded but spirix returns Zero
//! (and symmetric for Vanished → Zero).
use spirix::*;

type S = ScalarF3E3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Class { Zero, Vanished, Normal, Exploded, Infinity, Undefined }

fn classify(s: S) -> Class {
    if s.is_undefined() { Class::Undefined }
    else if s.is_zero() { Class::Zero }
    else if s.is_infinite() { Class::Infinity }
    else if s.exploded() { Class::Exploded }
    else if s.vanished() { Class::Vanished }
    else { Class::Normal }
}

fn show(label: &str, s: S) {
    let b = unsafe { std::mem::transmute::<S, [i8; 2]>(s) };
    let f: f64 = s.into();
    println!("  {:>10}: [0x{:02X},{:>4}] {:?} f64={:e}",
             label, b[0] as u8, b[1], classify(s), f);
}

fn main() {
    // Enumerate all F3E3 normal × normal pairs and look for div results where a is Normal, b is Vanished-class, result should be Exploded but isn't.
    // Also Normal / Exploded → expected Vanished, but result is Zero.
    let mut miss_norm_van_nonxpl = 0;
    let mut miss_norm_xpl_nonvan = 0;
    let mut shown = 0;

    for fa in -128i8..=127 {
        for ea in -128i8..=127 {
            let a = unsafe { std::mem::transmute::<[i8; 2], S>([fa, ea]) };
            if classify(a) != Class::Normal { continue; }
            for fb in -128i8..=127 {
                for eb in -128i8..=127 {
                    let b = unsafe { std::mem::transmute::<[i8; 2], S>([fb, eb]) };
                    let cb = classify(b);
                    // We want normal / (vanished or exploded)
                    if !matches!(cb, Class::Vanished | Class::Exploded) { continue; }
                    let r = a / b;
                    let cr = classify(r);
                    let expected = match cb {
                        Class::Vanished => Class::Exploded, // finite / tiny = huge
                        Class::Exploded => Class::Vanished, // finite / huge = tiny
                        _ => unreachable!(),
                    };
                    if cr != expected {
                        if cb == Class::Vanished { miss_norm_van_nonxpl += 1; }
                        if cb == Class::Exploded { miss_norm_xpl_nonvan += 1; }
                        if shown < 6 {
                            println!("--- a/b ---");
                            show("a", a);
                            show("b", b);
                            show("a/b (spirix)", r);
                            println!("  expected class: {:?}", expected);
                            shown += 1;
                        }
                    }
                }
            }
        }
    }
    println!();
    println!("Normal / Vanished NOT exploded: {}", miss_norm_van_nonxpl);
    println!("Normal / Exploded NOT vanished: {}", miss_norm_xpl_nonvan);
}
