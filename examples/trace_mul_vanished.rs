use spirix::*;
type S = ScalarF3E3;

fn main() {
    let mut found = 0;
    'outer: for f1 in -128i8..=127 {
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
                    let r = s1 * s2;
                    if r.is_zero() {
                        let b1 = unsafe { std::mem::transmute::<S, [i8; 2]>(s1) };
                        let b2 = unsafe { std::mem::transmute::<S, [i8; 2]>(s2) };
                        let br = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
                        println!(
                            "[↓]={:#04x},{} × [#]={:#04x},{} → [0]={:#04x},{}",
                            b1[0] as u8, b1[1], b2[0] as u8, b2[1], br[0] as u8, br[1]
                        );
                        found += 1;
                        if found >= 5 {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    println!("found {} bad cases", found);
}
