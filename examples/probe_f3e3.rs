use spirix::*;

fn main() {
    let s = Scalar::<i8, i8> {
        fraction: 0,
        exponent: -1,
    };
    println!("fraction=0 exp=-1");
    println!("  is_normal:    {}", s.is_normal());
    println!("  is_zero:      {}", s.is_zero());
    println!("  exploded:     {}", s.exploded());
    println!("  vanished:     {}", s.vanished());
    println!("  is_undefined: {}", s.is_undefined());
    println!("  is_infinite:  {}", s.is_infinite());
    let f: f64 = s.into();
    println!("  as f64:       {}", f);
    println!("  display:      {}", s);

    println!();
    // Also check neighbors
    for ex in -3i8..=3 {
        let s = Scalar::<i8, i8> {
            fraction: 0,
            exponent: ex,
        };
        let f: f64 = s.into();
        println!(
            "fraction=0 exp={}: is_normal={} f64={}",
            ex,
            s.is_normal(),
            f
        );
    }
    println!();
    for fr in -2i8..=2 {
        let s = Scalar::<i8, i8> {
            fraction: fr,
            exponent: -1,
        };
        let f: f64 = s.into();
        println!(
            "fraction={} exp=-1: is_normal={} f64={}",
            fr,
            s.is_normal(),
            f
        );
    }

    println!("\n=== Asymmetry probe: fraction=-128 (the i8 sign-only bit) ===");
    for ex in -3i8..=3 {
        let s = Scalar::<i8, i8> {
            fraction: i8::MIN,
            exponent: ex,
        };
        let f: f64 = s.into();
        println!(
            "fraction=-128 exp={}: is_normal={} is_zero={} f64={}",
            ex,
            s.is_normal(),
            s.is_zero(),
            f
        );
    }

    println!("\n=== Full count: how many normals exist per exp bucket? ===");
    for ex in [-1i8, 0, 1, 127i8].iter() {
        let mut npos = 0;
        let mut nneg = 0;
        let mut nother = 0;
        for fr in i8::MIN..=i8::MAX {
            let s = Scalar::<i8, i8> {
                fraction: fr,
                exponent: *ex,
            };
            if !s.is_normal() {
                nother += 1;
                continue;
            }
            let f: f64 = s.into();
            if f > 0.0 {
                npos += 1;
            } else if f < 0.0 {
                nneg += 1;
            } else {
                nother += 1;
            }
        }
        println!(
            "exp={}: pos_normals={} neg_normals={} other={}",
            ex, npos, nneg, nother
        );
    }
}
