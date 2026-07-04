use spirix::*;
fn main() {
    let a5 = CircleF3E3::from((15i128, 25u32));
    let sum = a5 + 2f32;
    eprintln!(
        "(15+25i) + 2 = r={} i={}",
        sum.r().to_f64(),
        sum.i().to_f64()
    );
}
