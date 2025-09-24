use spirix::*;

fn main() {
    let val_e4 = ScalarF3E4::from(-4);
    let val_e5 = ScalarF3E5::from(-4);

    println!("ScalarF3E4(-4): {:?}", val_e4);
    println!("ScalarF3E5(-4): {:?}", val_e5);

    println!("E4 square: {:?}", val_e4.square());
    println!("E5 square: {:?}", val_e5.square());

    // Check exponent limits
    println!("E4 MAX_EXPONENT: {:?}", <i16 as spirix::ExponentConstants>::MAX_EXPONENT);
    println!("E5 MAX_EXPONENT: {:?}", <i32 as spirix::ExponentConstants>::MAX_EXPONENT);
    println!("E4 MIN_EXPONENT: {:?}", <i16 as spirix::ExponentConstants>::MIN_EXPONENT);
    println!("E5 MIN_EXPONENT: {:?}", <i32 as spirix::ExponentConstants>::MIN_EXPONENT);
}