use spirix::*;

fn show(name: &str, c: CircleF5E3) {
    let r: f32 = c.r().into();
    let i: f32 = c.i().into();
    println!("{} = ({}, {}) bits = {:?}", name, r, i, c);
}

fn main() {
    show("1/1     ", CircleF5E3::ONE / CircleF5E3::ONE);
    show("1.5/1   ", CircleF5E3::from(1.5f32) / CircleF5E3::ONE);
    show(
        "1.5/1.5 ",
        CircleF5E3::from(1.5f32) / CircleF5E3::from(1.5f32),
    );
    show("2/1     ", CircleF5E3::from(2.0f32) / CircleF5E3::ONE);
    show(
        "1/(1+i) ",
        CircleF5E3::ONE / CircleF5E3::from((1.0f32, 1.0f32)),
    );
    show(
        "4/2     ",
        CircleF5E3::from(4.0f32) / CircleF5E3::from(2.0f32),
    );
}
