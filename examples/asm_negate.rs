//! Exposes both scalar_negate variants at a concrete type so their asm is
//! visible in the compiled binary. Not a real program — just monomorphization
//! fodder for disassembly.
use spirix::ScalarF3E3;

#[inline(never)]
#[no_mangle]
pub fn negate_default(mut x: ScalarF3E3) -> ScalarF3E3 {
    x = -x; // Neg operator calls the default (branchy) scalar_negate
    x
}

#[inline(never)]
#[no_mangle]
pub fn negate_unified(mut x: ScalarF3E3) -> ScalarF3E3 {
    x.scalar_negate_unified();
    x
}

fn main() {
    let a = ScalarF3E3::from(1);
    println!("{:?}", negate_default(a).is_negative());
    println!("{:?}", negate_unified(a).is_negative());
}
