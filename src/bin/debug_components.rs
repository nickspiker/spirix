use spirix::*;

fn main() {
    println!("=== F3E5 Component Debug ===");
    let complex = CircleF3E5::from((-4i16, 0f32));

    let r = complex.r();
    let i = complex.i();

    println!("r(): {:?}", r);
    println!("i(): {:?}", i);

    let r_squared = r.square();
    let i_squared = i.square();

    println!("r.square(): {:?}", r_squared);
    println!("i.square(): {:?}", i_squared);

    let sum = r_squared + i_squared;
    println!("r.square() + i.square(): {:?}", sum);

    println!("\n=== F3E4 Component Debug ===");
    let complex_e4 = CircleF3E4::from((-4i16, 0f32));

    let r_e4 = complex_e4.r();
    let i_e4 = complex_e4.i();

    println!("r(): {:?}", r_e4);
    println!("i(): {:?}", i_e4);

    let r_squared_e4 = r_e4.square();
    let i_squared_e4 = i_e4.square();

    println!("r.square(): {:?}", r_squared_e4);
    println!("i.square(): {:?}", i_squared_e4);

    let sum_e4 = r_squared_e4 + i_squared_e4;
    println!("r.square() + i.square(): {:?}", sum_e4);
}