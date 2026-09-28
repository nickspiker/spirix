//! Differential test for the ports in `ports/`.
//!
//! Each port is required to be bit-identical to `CircleF5E4 / CircleF5E4` on every normal input, and to correctly report "escape class, did not handle it" on every non-normal one.
//!
//! Run with `cargo run --release` from `ports/test/`. Exits non-zero on any mismatch. The Swift port is skipped if `swiftc` is unavailable.

use spirix::CircleF5E4;

/// `#[repr(C)]` mirror of `Circle<i32, i16>`.
///
/// `spirix::Circle` is not `#[repr(C)]`, so its layout is not guaranteed by the language even though it happens to match today (size 12, align 4, offsets 0/4/8). The ports are written against this layout, so cross here explicitly rather than transmuting and hoping.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
struct CircleAbi {
    real: i32,
    imaginary: i32,
    exponent: i16,
}

const POISON: CircleAbi = CircleAbi {
    real: 0x5A5A_5A5A,
    imaginary: 0x5A5A_5A5A,
    exponent: 0x5A5A,
};

impl From<CircleF5E4> for CircleAbi {
    fn from(c: CircleF5E4) -> Self {
        CircleAbi {
            real: c.real,
            imaginary: c.imaginary,
            exponent: c.exponent,
        }
    }
}

impl CircleAbi {
    fn to_spirix(self) -> CircleF5E4 {
        CircleF5E4 {
            real: self.real,
            imaginary: self.imaginary,
            exponent: self.exponent,
        }
    }
}

/// Every port exposes the same C signature: write `*out`, return whether it handled the operands.
type Port = unsafe extern "C" fn(*mut CircleAbi, *const CircleAbi, *const CircleAbi) -> bool;

extern "C" {
    fn circle_divide_f5e4(out: *mut CircleAbi, n: *const CircleAbi, d: *const CircleAbi) -> bool;
    fn circle_divide_struct(out: *mut CircleAbi, n: *const CircleAbi, d: *const CircleAbi) -> bool;
    #[cfg(have_swift)]
    fn circle_divide_swift(out: *mut CircleAbi, n: *const CircleAbi, d: *const CircleAbi) -> bool;
}

fn ports() -> Vec<(&'static str, Port)> {
    // `mut` is only used by the Swift push below, which cfg's out without it.
    #[cfg_attr(not(have_swift), allow(unused_mut))]
    let mut v: Vec<(&'static str, Port)> = vec![
        ("llvm-ir    ", circle_divide_f5e4 as Port),
        ("objective-c", circle_divide_struct as Port),
    ];
    #[cfg(have_swift)]
    v.push(("swift      ", circle_divide_swift as Port));
    v
}

/// Same LCG the crate's own fuzz tests use, so runs are reproducible and the harness stays dependency-free.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 1
    }
    fn i32(&mut self) -> i32 {
        self.next() as i32
    }
    /// Any exponent except AMBIG (0), i.e. any normal one.
    fn normal_exp(&mut self) -> i16 {
        let e = self.next() as u16;
        if e == 0 {
            1
        } else {
            e as i16
        }
    }
}

fn c(real: i32, imaginary: i32, exponent: i16) -> CircleF5E4 {
    CircleF5E4 {
        real,
        imaginary,
        exponent,
    }
}

/// The shared corpus. Every port sees exactly these pairs.
fn corpus() -> Vec<(&'static str, CircleF5E4, CircleF5E4)> {
    let mut v = Vec::new();

    // 1. Randomized normal pairs with fully arbitrary components. Random components are rarely canonical N1, so this leans on canonicalization.
    let mut r = Lcg(0x51D1_0DED_C0FF_EE01);
    for _ in 0..200_000 {
        v.push((
            "random",
            c(r.i32(), r.i32(), r.normal_exp()),
            c(r.i32(), r.i32(), r.normal_exp()),
        ));
    }

    // 2. Values built through the public API: already-canonical components.
    for i in -60i32..=60 {
        for j in -60i32..=60 {
            if i == 0 && j == 0 {
                continue;
            }
            let a = CircleF5E4::from((i as f32, (j as f32) * 0.5));
            let b = CircleF5E4::from((j as f32, (i as f32) * 0.25));
            if a.is_normal() && b.is_normal() {
                v.push(("api", a, b));
            }
        }
    }

    // 3. The two's-complement boundary the pre-halving guard exists for: i32::MIN in either component of either operand, both ways round.
    let bound = [
        i32::MIN,
        i32::MIN + 1,
        i32::MAX,
        -1,
        0,
        1,
        1 << 30,
        -(1 << 30),
    ];
    for &ar in &bound {
        for &ai in &bound {
            for &br in &bound {
                for &bi in &bound {
                    for &e in &[1i16, -1, 0x4000, -0x4000, i16::MIN, i16::MAX] {
                        v.push(("boundary", c(ar, ai, e), c(br, bi, e)));
                    }
                }
            }
        }
    }

    // 4. Exponent saturation in both directions: cycle positions near the ends of the in-range window [1, 65535].
    let exps: [i16; 10] = [1, 2, 3, i16::MAX, i16::MIN, -3, -2, -1, 0x4000, -0x4000];
    for &ea in &exps {
        for &eb in &exps {
            for &(r, i) in &[
                (1 << 30, 0),
                (0, 1 << 30),
                (1 << 30, 1 << 30),
                (-(1 << 30), 1),
                (i32::MAX, i32::MIN),
            ] {
                v.push(("saturation", c(r, i, ea), c(i, r, eb)));
            }
        }
    }

    // 5. Degenerate normals: zero components at a normal exponent, reachable because Circle's fields are pub. 0-valued/x = ZERO, x/0-valued = INF.
    for &e in &exps {
        v.push(("degenerate", c(0, 0, e), c(1 << 30, 0, e)));
        v.push(("degenerate", c(1 << 30, 0, e), c(0, 0, e)));
        v.push(("degenerate", c(0, 0, e), c(0, 0, e)));
    }

    // 6. Escape classes: must be declined, not answered. Everything at AMBIG.
    let escapes = [
        CircleF5E4::ZERO,
        CircleF5E4::INFINITY,
        c(1 << 30, 1 << 30, 0),    // N1 -> exploded
        c(1 << 29, 1 << 29, 0),    // N2 -> vanished
        c(0x1234_5678, 0x0BAD, 0), // mismatched junk -> undefined
    ];
    let normal = c(1 << 30, 0, 0x4000);
    for &a in &escapes {
        for &b in &escapes {
            v.push(("escape", a, b));
        }
        v.push(("escape", a, normal));
        v.push(("escape", normal, a));
    }

    v
}

fn main() {
    let cases = corpus();
    let ports = ports();
    println!("{} cases x {} ports\n", cases.len(), ports.len());

    #[cfg(not(have_swift))]
    println!("note: swiftc unavailable, Swift port skipped\n");

    let mut total_fail = 0usize;
    for (name, f) in &ports {
        let mut fails = 0usize;
        let mut shown = 0usize;
        for &(label, num, den) in &cases {
            let (n, d) = (CircleAbi::from(num), CircleAbi::from(den));
            let mut out = POISON;
            let handled = unsafe { f(&mut out, &n, &d) };
            let normal = num.is_normal() && den.is_normal();

            if handled != normal {
                fails += 1;
                if shown < 3 {
                    shown += 1;
                    let verb = if handled { "handled" } else { "declined" };
                    println!("  [{label}] {verb} a pair it should not have: {num:?} / {den:?}");
                }
                continue;
            }
            if !handled {
                // Declining means writing nothing at all.
                if out != POISON {
                    fails += 1;
                    if shown < 3 {
                        shown += 1;
                        println!("  [{label}] declined but still wrote to *out: {out:?}");
                    }
                }
                continue;
            }
            let want = CircleAbi::from(num / den);
            if out != want {
                fails += 1;
                if shown < 3 {
                    shown += 1;
                    println!(
                        "  [{label}] mismatch\n    num  = {num:?}\n    den  = {den:?}\n    rust = {want:?}  ({})\n    port = {out:?}  ({})",
                        want.to_spirix(),
                        out.to_spirix()
                    );
                }
            }
        }
        println!("{name}: {fails} mismatches");
        total_fail += fails;
    }

    if total_fail != 0 {
        println!("\n{total_fail} total mismatches");
        std::process::exit(1);
    }
    println!("\nall ports match the Rust implementation bit-for-bit.");
}
