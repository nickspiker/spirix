use spirix::*;
type S = ScalarF3E3;

fn class(s: S) -> &'static str {
    if s.is_undefined() {
        "[℘?]"
    } else if s.is_zero() {
        "[0]"
    } else if s.is_infinite() {
        "[∞]"
    } else if s.exploded() {
        "[↑]"
    } else if s.vanished() {
        "[↓]"
    } else if s.is_normal() {
        "[#]"
    } else {
        "??"
    }
}

fn main() {
    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !s1.vanished() {
                continue;
            }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !s2.is_normal() {
                        continue;
                    }
                    // Test [#] × [↓] (col × row per the table convention)
                    let r = s2 * s1;
                    if r.is_zero() {
                        let s1f: f64 = s1.into();
                        let s2f: f64 = s2.into();
                        let s1b = unsafe { std::mem::transmute::<S, [i8; 2]>(s1) };
                        let s2b = unsafe { std::mem::transmute::<S, [i8; 2]>(s2) };
                        println!("[↓]×[#] → [0] bug:");
                        println!(
                            "  [↓]: frac={:#010b} exp={} ({})",
                            s1b[0] as u8, s1b[1], s1f
                        );
                        println!(
                            "  [#]: frac={:#010b} exp={} ({})",
                            s2b[0] as u8, s2b[1], s2f
                        );
                        println!("  class: {}", class(r));
                        return;
                    }
                }
            }
        }
    }
    println!("no bug found");
}
