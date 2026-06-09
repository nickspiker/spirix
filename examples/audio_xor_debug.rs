use spirix::ScalarF4E3;

fn show(label: &str, x: ScalarF4E3) {
    let v: f32 = (&x).into();
    println!(
        "  {:<6} = (frac={:>6} = {:>04x}, exp={:>3} = {:>02x})  -> f32 = {}",
        label, x.fraction, x.fraction as u16, x.exponent, x.exponent as u8, v
    );
}

fn main() {
    for &(l, r) in &[(4i16, 5i16), (16i16, 17i16), (1i16, 2i16), (-2i16, 1i16)] {
        println!("--- L={}, R={}, native L^R = {} ---", l, r, l ^ r);
        let ls = ScalarF4E3::from(l as f32);
        let rs = ScalarF4E3::from(r as f32);
        show("L", ls);
        show("R", rs);
        let xs = ls ^ rs;
        show("L^R", xs);
        let xf: f32 = (&xs).into();
        println!("  rounded-to-i16: {}", xf.round() as i16);
    }
}
