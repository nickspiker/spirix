use spirix::*;

fn fr(c: CircleF5E3) -> (f32, f32) {
    (c.r().into(), c.i().into())
}

fn main() {
    let z = CircleF5E3::from((1.0f32, 1.0f32));
    let one_c = CircleF5E3::ONE;
    let one_s = ScalarF5E3::ONE;

    let q1 = one_c / z;
    let q2 = one_s / z;
    let r = z.reciprocal();

    println!("z = (1, 1)");
    let (q1r, q1i) = fr(q1);
    let (q2r, q2i) = fr(q2);
    let (rr, ri) = fr(r);
    println!("CircleONE / z  = ({}, {})  bits = {:?}", q1r, q1i, q1);
    println!("ScalarONE / z  = ({}, {})  bits = {:?}", q2r, q2i, q2);
    println!("z.reciprocal() = ({}, {})  bits = {:?}", rr, ri, r);
    println!("expected: (0.5, -0.5)");
}
