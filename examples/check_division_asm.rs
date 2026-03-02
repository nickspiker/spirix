// Check what assembly is generated for division
#![allow(unused)]

#[inline(never)]
pub fn div_u32(x: u32, y: u32) -> u32 {
    x / y
}

#[inline(never)]
pub fn div_u64(x: u64, y: u64) -> u64 {
    x / y
}

fn main() {
    let x: u32 = 12345678;
    let y: u32 = 456;
    println!("u32: {} / {} = {}", x, y, div_u32(x, y));

    let x: u64 = 123456789012;
    let y: u64 = 4567;
    println!("u64: {} / {} = {}", x, y, div_u64(x, y));
}
