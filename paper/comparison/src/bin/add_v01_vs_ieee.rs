//! Spirix v0.1 N0 add vs IEEE 754 binary32. Phase A (random IEEE inputs) and Phase B (random Spirix inputs) sequential.
//!
//! The bit-accurate algorithm is the unified add/sub kernel `spirix_addsub_normal_normal` in [paper-comparison lib](../lib.rs) — same datapath as the silicon Verilog's add/sub unit, with a `sub: bool` selector that integer-negates the second operand at the alignment input. For add, the binary calls it with `sub=false`.

use spirix_paper_comparison::{run_phase_a, run_phase_b, spirix_addsub, FRAC};

fn main() {
    const N: u64 = 10_000_000;
    println!("Spirix v0.1 N0 add (FRAC={FRAC}, EXP=8, banker's rounding) vs IEEE 754 binary32");
    println!();

    run_phase_a(
        "add",
        N,
        0xDEAD_BEEF_CAFE_1234,
        |a, b| a + b,
        |a, b| spirix_addsub(a, b, false),
    );
    println!();
    run_phase_b(
        "add",
        N,
        0xCAFE_BABE_DEAD_5678,
        |a, b| a + b,
        |a, b| spirix_addsub(a, b, false),
    );
}
