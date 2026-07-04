use spirix::*;

#[derive(Clone, Copy, PartialEq)]
enum Cls {
    Z,
    V,
    N,
    E,
    Inf,
    U,
}
fn lab(c: Cls) -> &'static str {
    match c {
        Cls::Z => "[0]",
        Cls::V => "[↓]",
        Cls::N => "[#]",
        Cls::E => "[↑]",
        Cls::Inf => "[∞]",
        Cls::U => "[℘]",
    }
}
fn classify_c(c: &CircleF3E3) -> Cls {
    if c.is_undefined() {
        Cls::U
    } else if c.is_zero() {
        Cls::Z
    } else if c.is_infinite() {
        Cls::Inf
    } else if c.exploded() {
        Cls::E
    } else if c.vanished() {
        Cls::V
    } else {
        Cls::N
    }
}
fn classify_s(s: &ScalarF3E3) -> Cls {
    if s.is_undefined() {
        Cls::U
    } else if s.is_zero() {
        Cls::Z
    } else if s.is_infinite() {
        Cls::Inf
    } else if s.exploded() {
        Cls::E
    } else if s.vanished() {
        Cls::V
    } else {
        Cls::N
    }
}
fn circles() -> Vec<(Cls, CircleF3E3)> {
    let z = CircleF3E3::ZERO;
    vec![
        (Cls::Z, z),
        (Cls::V, CircleF3E3::MIN_POS / 4u16),
        (Cls::N, CircleF3E3::from((3.0f32, 4.0))),
        (Cls::E, CircleF3E3::MAX * CircleF3E3::from((2.0f32, 0.0))),
        (Cls::Inf, CircleF3E3::INFINITY),
        (Cls::U, z / z),
    ]
}
fn scalars() -> Vec<(Cls, ScalarF3E3)> {
    vec![
        (Cls::Z, ScalarF3E3::ZERO),
        (Cls::V, ScalarF3E3::VANISHED_POS),
        (Cls::N, ScalarF3E3::from(3)),
        (Cls::E, ScalarF3E3::EXPLODED_POS),
        (Cls::Inf, ScalarF3E3::INFINITY),
        (Cls::U, ScalarF3E3::ZERO / ScalarF3E3::ZERO),
    ]
}
// Div table from Spirix Scalar truth tables: a / b
fn div_table() -> [[Vec<Cls>; 6]; 6] {
    use Cls::*;
    let z = vec![Z];
    let v = vec![V];
    let _n = vec![N];
    let e = vec![E];
    let inf = vec![Inf];
    let u = vec![U];
    let any_nve = vec![Z, V, N, E];
    [
        // a=Z: 0/X = 0 for X!=0, 0/0=u, 0/inf=0, 0/u=u
        [
            u.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=V: van/0=inf, van/van=u, van/n=van, van/e=van, van/inf=0, van/u=u
        [
            inf.clone(),
            u.clone(),
            v.clone(),
            v.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=N: n/0=inf, n/van=exp, n/n=any_nve, n/e=van, n/inf=0, n/u=u
        [
            inf.clone(),
            e.clone(),
            any_nve.clone(),
            v.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=E: exp/0=inf, exp/van=exp, exp/n=exp, exp/exp=u, exp/inf=0, exp/u=u
        [
            inf.clone(),
            e.clone(),
            e.clone(),
            u.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=Inf: inf/0=inf, inf/X=inf for X!=0, inf/inf=u
        [
            inf.clone(),
            inf.clone(),
            inf.clone(),
            inf.clone(),
            u.clone(),
            u.clone(),
        ],
        // a=U: all undefined
        [
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
    ]
}
// AND table from and_truth_table (positive operands). [0] absorbs, [∞] identity, escape & escape: same → U; ↓&↑ → Z|E; escape & normal → U; #&# → N|Z|V; U propagates.
fn and_table() -> [[Vec<Cls>; 6]; 6] {
    use Cls::*;
    let z = vec![Z];
    let v = vec![V];
    let _n = vec![N];
    let e = vec![E];
    let inf = vec![Inf];
    let u = vec![U];
    let nzv = vec![N, Z, V];
    let ze = vec![Z, E];
    [
        // a=Z: absorber
        [
            z.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=V: V&0=Z, V&V=U, V&N=U, V&E=Z|E (same-sign), V&Inf=V, V&U=U
        [
            z.clone(),
            u.clone(),
            u.clone(),
            ze.clone(),
            v.clone(),
            u.clone(),
        ],
        // a=N: N&0=Z, N&V=U, N&N=N|Z|V, N&E=U, N&Inf=N, N&U=U
        [
            z.clone(),
            u.clone(),
            nzv.clone(),
            u.clone(),
            vec![N],
            u.clone(),
        ],
        // a=E: E&0=Z, E&V=Z|E, E&N=U, E&E=U, E&Inf=E, E&U=U
        [
            z.clone(),
            ze.clone(),
            u.clone(),
            u.clone(),
            e.clone(),
            u.clone(),
        ],
        // a=Inf: identity for non-Z (Inf&Inf=Inf)
        [
            z.clone(),
            v.clone(),
            vec![N],
            e.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=U: propagates
        [
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
    ]
}
// OR table from or_truth_table. [0] identity, [∞] absorber; escape | escape: same → U; ↓|↑ → V|E; escape | normal → U; #|# → N|V; U propagates.
fn or_table() -> [[Vec<Cls>; 6]; 6] {
    use Cls::*;
    let z = vec![Z];
    let v = vec![V];
    let _n = vec![N];
    let e = vec![E];
    let inf = vec![Inf];
    let u = vec![U];
    let nv = vec![N, V];
    let ve = vec![V, E];
    [
        // a=Z: identity
        [
            z.clone(),
            v.clone(),
            vec![N],
            e.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=V: V|0=V, V|V=U, V|N=U, V|E=V|E, V|Inf=Inf, V|U=U
        [
            v.clone(),
            u.clone(),
            u.clone(),
            ve.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=N: N|0=N, N|V=U, N|N=N|V, N|E=U, N|Inf=Inf, N|U=U
        [
            vec![N],
            u.clone(),
            nv.clone(),
            u.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=E: E|0=E, E|V=V|E, E|N=U, E|E=U, E|Inf=Inf, E|U=U
        [
            e.clone(),
            ve.clone(),
            u.clone(),
            u.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=Inf: absorber
        [
            inf.clone(),
            inf.clone(),
            inf.clone(),
            inf.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=U: propagates
        [
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
    ]
}
// XOR table from xor_truth_table. [0] identity, [∞] inverts; same-class escape → U (even bit-identical); ↓⊻↑ → E; escape⊻normal → U; #⊻# (same-sign positive) → Z (self-cancel for our positive operands).
fn xor_table() -> [[Vec<Cls>; 6]; 6] {
    use Cls::*;
    let z = vec![Z];
    let v = vec![V];
    let _n = vec![N];
    let e = vec![E];
    let inf = vec![Inf];
    let u = vec![U];
    [
        // a=Z: identity
        [
            z.clone(),
            v.clone(),
            vec![N],
            e.clone(),
            inf.clone(),
            u.clone(),
        ],
        // a=V: V^0=V, V^V=U, V^N=U, V^E=E, V^Inf=V (inverts), V^U=U
        [
            v.clone(),
            u.clone(),
            u.clone(),
            e.clone(),
            v.clone(),
            u.clone(),
        ],
        // a=N: N^0=N, N^V=U, N^N (same-sign positive)=Z, N^E=U, N^Inf=N, N^U=U
        [vec![N], u.clone(), z.clone(), u.clone(), vec![N], u.clone()],
        // a=E: E^0=E, E^V=E, E^N=U, E^E=U, E^Inf=E, E^U=U
        [
            e.clone(),
            e.clone(),
            u.clone(),
            u.clone(),
            e.clone(),
            u.clone(),
        ],
        // a=Inf: inverts to other class; Inf^Inf=Z
        [
            inf.clone(),
            v.clone(),
            vec![N],
            e.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=U: propagates
        [
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
    ]
}
// Mod table (using positive operands only — same-sign branches). Rules from scalar_modulus_scalar: zero anywhere (not U) → Z; transfinite numerator → U; vanished period → U; infinite period → U; exploded period & same sign → numerator; vanished numerator & normal period & same sign → numerator
fn mod_table() -> [[Vec<Cls>; 6]; 6] {
    use Cls::*;
    let z = vec![Z];
    let v = vec![V];
    let n = vec![N];
    let _e = vec![E];
    let _inf = vec![Inf];
    let u = vec![U];
    let zvn = vec![Z, V, N];
    [
        // a=Z: 0%X = 0 for X!=U, 0%U=U
        [
            z.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            z.clone(),
            u.clone(),
        ],
        // a=V: V%0=Z, V%V=U, V%N=V (same sign), V%E=V (same sign), V%Inf=U, V%U=U
        [
            z.clone(),
            u.clone(),
            v.clone(),
            v.clone(),
            u.clone(),
            u.clone(),
        ],
        // a=N: N%0=Z, N%V=U, N%N=Z|V|N, N%E=N (same sign), N%Inf=U, N%U=U
        [
            z.clone(),
            u.clone(),
            zvn.clone(),
            n.clone(),
            u.clone(),
            u.clone(),
        ],
        // a=E: E%0=Z, E%X=U for X!=Z
        [
            z.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
        // a=Inf: Inf%0=Z, Inf%X=U for X!=Z
        [
            z.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
        // a=U: all U
        [
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
            u.clone(),
        ],
    ]
}
fn check_op_c<A: Copy, B: Copy, F: Fn(A, B) -> CircleF3E3>(
    name: &str,
    a_list: &[(Cls, A)],
    b_list: &[(Cls, B)],
    op: F,
    table: &[[Vec<Cls>; 6]; 6],
) {
    let idx = |c: Cls| match c {
        Cls::Z => 0,
        Cls::V => 1,
        Cls::N => 2,
        Cls::E => 3,
        Cls::Inf => 4,
        Cls::U => 5,
    };
    let mut fails = 0;
    for (ac, a) in a_list {
        for (bc, b) in b_list {
            let r = op(*a, *b);
            let rc = classify_c(&r);
            let expected = &table[idx(*ac)][idx(*bc)];
            if !expected.contains(&rc) {
                let exp_str: Vec<&str> = expected.iter().map(|c| lab(*c)).collect();
                println!(
                    "  ✗ {}: {} {} {} = {} (expected {:?})",
                    name,
                    lab(*ac),
                    name,
                    lab(*bc),
                    lab(rc),
                    exp_str
                );
                fails += 1;
            }
        }
    }
    println!("{}: {} mismatches", name, fails);
}
fn check_op_s<A: Copy, B: Copy, F: Fn(A, B) -> ScalarF3E3>(
    name: &str,
    a_list: &[(Cls, A)],
    b_list: &[(Cls, B)],
    op: F,
    table: &[[Vec<Cls>; 6]; 6],
) {
    let idx = |c: Cls| match c {
        Cls::Z => 0,
        Cls::V => 1,
        Cls::N => 2,
        Cls::E => 3,
        Cls::Inf => 4,
        Cls::U => 5,
    };
    let mut fails = 0;
    for (ac, a) in a_list {
        for (bc, b) in b_list {
            let r = op(*a, *b);
            let rc = classify_s(&r);
            let expected = &table[idx(*ac)][idx(*bc)];
            if !expected.contains(&rc) {
                let exp_str: Vec<&str> = expected.iter().map(|c| lab(*c)).collect();
                println!(
                    "  ✗ {}: {} {} {} = {} (expected {:?})",
                    name,
                    lab(*ac),
                    name,
                    lab(*bc),
                    lab(rc),
                    exp_str
                );
                fails += 1;
            }
        }
    }
    println!("{}: {} mismatches", name, fails);
}
fn main() {
    let cs = circles();
    let ss = scalars();
    let div_t = div_table();
    let mod_t = mod_table();

    println!("--- Circle / Circle ---");
    check_op_c("/", &cs, &cs, |a: CircleF3E3, b: CircleF3E3| a / b, &div_t);
    println!("--- Circle / Scalar ---");
    check_op_c("/", &cs, &ss, |a: CircleF3E3, b: ScalarF3E3| a / b, &div_t);
    println!("--- Scalar / Circle ---");
    check_op_c("/", &ss, &cs, |a: ScalarF3E3, b: CircleF3E3| a / b, &div_t);

    println!("\n--- Scalar % Scalar (baseline) ---");
    check_op_s("%", &ss, &ss, |a: ScalarF3E3, b: ScalarF3E3| a % b, &mod_t);
    println!("--- Circle % Circle ---");
    check_op_c("%", &cs, &cs, |a: CircleF3E3, b: CircleF3E3| a % b, &mod_t);
    println!("--- Circle % Scalar (returns Scalar) ---");
    check_op_s("%", &cs, &ss, |a: CircleF3E3, b: ScalarF3E3| a % b, &mod_t);
    println!("--- Scalar % Circle ---");
    check_op_c("%", &ss, &cs, |a: ScalarF3E3, b: CircleF3E3| a % b, &mod_t);

    let and_t = and_table();
    let or_t = or_table();
    let xor_t = xor_table();
    println!("\n--- Scalar & Scalar (baseline) ---");
    check_op_s("&", &ss, &ss, |a: ScalarF3E3, b: ScalarF3E3| a & b, &and_t);
    println!("--- Circle & Circle ---");
    check_op_c("&", &cs, &cs, |a: CircleF3E3, b: CircleF3E3| a & b, &and_t);
    println!("\n--- Scalar | Scalar (baseline) ---");
    check_op_s("|", &ss, &ss, |a: ScalarF3E3, b: ScalarF3E3| a | b, &or_t);
    println!("--- Circle | Circle ---");
    check_op_c("|", &cs, &cs, |a: CircleF3E3, b: CircleF3E3| a | b, &or_t);
    println!("\n--- Scalar ^ Scalar (baseline) ---");
    check_op_s("^", &ss, &ss, |a: ScalarF3E3, b: ScalarF3E3| a ^ b, &xor_t);
    println!("--- Circle ^ Circle ---");
    check_op_c("^", &cs, &cs, |a: CircleF3E3, b: CircleF3E3| a ^ b, &xor_t);
}
