//! Spirix v0.1 N0 sub vs IEEE 754 binary32. Phase A (random IEEE inputs) and Phase B (random Spirix inputs) sequential.
//!
//! Subtraction uses the unified add/sub kernel `spirix_addsub_normal_normal` in [paper-comparison lib](../lib.rs) with `sub=true`, mirroring the silicon Verilog's `subOp`-flag-into-shared-datapath pattern. The integer-level negation of the second operand happens at the alignment input — no state-level `spirix_negate` dependency, no branch explosion across non-normal states.
//!
//! The truth table at `spirix_addsub` handles the cases that need state-aware negation (e.g., `0 - X` requires negating X) inline, with simple phase-flips (Exploded/Vanished) or compute-Q negation with boundary handling (Normal). MIN/MAX exponent boundaries saturate cleanly to vanished/exploded — no MIN-dumped-off-a-cliff issues.

use spirix_paper_comparison::{run_phase_a, run_phase_b, spirix_addsub, FRAC};

fn main() {
    const N: u64 = 10_000_000;
    println!(
        "Spirix v0.1 N0 sub (FRAC={FRAC}, EXP=8, banker's rounding, unified add/sub kernel with sub=true) vs IEEE 754 binary32"
    );
    println!();

    run_phase_a(
        "sub",
        N,
        0x5AB5_5AB5_5AB5_5AB5,
        |a, b| a - b,
        |a, b| spirix_addsub(a, b, true),
    );
    println!();
    run_phase_b(
        "sub",
        N,
        0xBABE_F00D_DEAD_5AB1,
        |a, b| a - b,
        |a, b| spirix_addsub(a, b, true),
    );
}
