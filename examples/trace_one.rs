use spirix::*;
use i256::I256;
type S = ScalarF3E3;

fn main() {
    let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([-64i8, -128]) };  // 0xC0 vanished
    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([0i8, -127]) };    // 0x00 NEG_ONE_NORMAL
    println!("s1 vanished={}, is_normal={}, is_zero={}", s1.vanished(), s1.is_normal(), s1.is_zero());
    println!("s2 vanished={}, is_normal={}, is_zero={}", s2.vanished(), s2.is_normal(), s2.is_zero());
    let r = s1 * s2;
    let br = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
    println!("result: [{:#04x}, {}]", br[0] as u8, br[1]);
    println!("  is_zero={}, is_vanished={}, is_normal={}", r.is_zero(), r.vanished(), r.is_normal());

    // Manually do the abnormal path logic
    let s1_stored: i8 = -64;  // 0xC0
    let s2_stored: i8 = 0;
    // inflate(s1, false) = sign_extend
    let s1_wide: i16 = s1_stored as i16;  // -64
    // inflate(s2, true) = XOR
    let mask: i16 = (-1i16) << 8;  // 0xFF00 = -256
    let s2_wide: i16 = (s2_stored as i16) ^ mask;  // -256
    println!("s1_wide={} (0x{:04x}), s2_wide={} (0x{:04x})", s1_wide, s1_wide as u16, s2_wide, s2_wide as u16);
    let product: i16 = s1_wide.wrapping_mul(s2_wide);
    println!("product={} (0x{:04x})", product, product as u16);
    let leading_same = product.leading_ones().max(product.leading_zeros()) as i32;
    println!("leading_same={}", leading_same);
}
