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
    let mut found_zero = None;
    let mut found_inf = None;
    let mut found_und = None;
    'outer: for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !s1.is_normal() {
                continue;
            }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !s2.is_normal() {
                        continue;
                    }
                    let r = s2 * s1;
                    let rc = class(r);
                    if rc == "[0]" && found_zero.is_none() {
                        found_zero = Some((s1, s2));
                    }
                    if rc == "[∞]" && found_inf.is_none() {
                        found_inf = Some((s1, s2));
                    }
                    if rc == "[℘?]" && found_und.is_none() {
                        found_und = Some((s1, s2));
                    }
                    if found_zero.is_some() && found_inf.is_some() && found_und.is_some() {
                        break 'outer;
                    }
                }
            }
        }
    }
    for (name, opt) in [
        ("ZERO", found_zero),
        ("INFTY", found_inf),
        ("UNDEF", found_und),
    ] {
        if let Some((s1, s2)) = opt {
            let b1 = unsafe { std::mem::transmute::<S, [i8; 2]>(s1) };
            let b2 = unsafe { std::mem::transmute::<S, [i8; 2]>(s2) };
            let f1: f64 = s1.into();
            let f2: f64 = s2.into();
            let r = s2 * s1;
            let br = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
            let fr: f64 = r.into();
            println!(
                "{}: s2*s1 = [{:#04x},{}] * [{:#04x},{}]  = [{:#04x},{}]",
                name, b2[0] as u8, b2[1], b1[0] as u8, b1[1], br[0] as u8, br[1]
            );
            println!(
                "      f64: {:.3e} * {:.3e} = {:.3e}  (spirix: {} = {:.3e})",
                f2,
                f1,
                f2 * f1,
                class(r),
                fr
            );
        } else {
            println!("{}: not found", name);
        }
    }
}
