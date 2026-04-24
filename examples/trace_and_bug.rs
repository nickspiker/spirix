use spirix::*;

type S = ScalarF3E3;

fn class(s: S) -> &'static str {
    if s.is_undefined() { "[℘?]" }
    else if s.is_zero() { "[0]" }
    else if s.is_infinite() { "[∞]" }
    else if s.exploded() { "[↑]" }
    else if s.vanished() { "[↓]" }
    else if s.is_normal() { "[#]" }
    else { "??" }
}

fn main() {
    // Find the smallest failing case by scanning all pairs
    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !s1.is_normal() { continue; }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !s2.is_normal() { continue; }
                    let r = s1 & s2;
                    if r.exploded() {
                        let f1v: f64 = s1.into();
                        let f2v: f64 = s2.into();
                        let rv: f64 = r.into();
                        let rf = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
                        println!("BUG: s1={:#010b} exp={} ({}) &", f1 as u8, e1, f1v);
                        println!("     s2={:#010b} exp={} ({})", f2 as u8, e2, f2v);
                        println!("     r.fraction={:#010b} r.exp={} class={} rv={}",
                                 rf[0] as u8, rf[1], class(r), rv);
                        return;
                    }
                }
            }
        }
    }
}
