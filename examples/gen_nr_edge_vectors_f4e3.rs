/// Generate F4E3 edge case test vectors for NR div/sqrt validation. Tests: undefined passthrough, zero, infinity, negative sqrt, exploded, vanished, and all cross-combinations. Output lines: mode a_frac a_exp b_frac b_exp result_frac result_exp (hex) mode: 0=divide, 1=sqrt
use spirix::Scalar;

fn emit_div(a_frac: i16, a_exp: i8, b_frac: i16, b_exp: i8) {
    let a = Scalar::<i16, i8> {
        fraction: a_frac,
        exponent: a_exp,
    };
    let b = Scalar::<i16, i8> {
        fraction: b_frac,
        exponent: b_exp,
    };
    let r = a / b;
    println!(
        "0 {:04x} {:02x} {:04x} {:02x} {:04x} {:02x}",
        a_frac as u16, a_exp as u8, b_frac as u16, b_exp as u8, r.fraction as u16, r.exponent as u8,
    );
}

fn emit_line_div(a_frac: i16, a_exp: i8, b_frac: i16, b_exp: i8, r_frac: i16, r_exp: i8) {
    println!(
        "0 {:04x} {:02x} {:04x} {:02x} {:04x} {:02x}",
        a_frac as u16, a_exp as u8, b_frac as u16, b_exp as u8, r_frac as u16, r_exp as u8,
    );
}

fn emit_sqrt(a_frac: i16, a_exp: i8) {
    let a = Scalar::<i16, i8> {
        fraction: a_frac,
        exponent: a_exp,
    };
    let r = a.sqrt();
    println!(
        "1 {:04x} {:02x} 0000 00 {:04x} {:02x}",
        a_frac as u16, a_exp as u8, r.fraction as u16, r.exponent as u8,
    );
}

fn main() {
    let ambig: i8 = -128;
    let mut count = 0u32;

    // Zero state: (frac=0, exp=AMBIG)
    let zero_f: i16 = 0;
    let zero_e: i8 = ambig;
    // Infinity state: (frac=-1 = all ones, exp=AMBIG)
    let inf_f: i16 = -1;
    let inf_e: i8 = ambig;

    let normals: [(i16, i8); 8] = [
        (0x4000, 0),
        (0x6000, 3),
        (-0x4000, 1),
        (-0x6000, -2),
        (0x7FFF, 126),
        (-0x7FFF, 126),
        (0x4001, -126),
        (-0x4001, -126),
    ];

    let undef_fracs: [i16; 4] = [
        0x1600u16 as i16,
        0x0800u16 as i16,
        0xE900u16 as i16,
        0xF600u16 as i16,
    ];

    let exploded_fracs: [i16; 2] = [0x5000, 0xB000u16 as i16];
    let vanished_fracs: [i16; 2] = [0x2000, 0xC000u16 as i16];

    // === DIVIDE ===

    // Zero / Zero
    emit_div(zero_f, zero_e, zero_f, zero_e);
    count += 1;

    // Normal / Zero → Infinity
    for &(nf, ne) in &normals {
        emit_div(nf, ne, zero_f, zero_e);
        count += 1;
    }

    // Infinity / Normal → Infinity
    for &(nf, ne) in &normals {
        emit_div(inf_f, inf_e, nf, ne);
        count += 1;
    }

    // Infinity / Infinity
    emit_div(inf_f, inf_e, inf_f, inf_e);
    count += 1;

    // Zero / Normal → Zero
    for &(nf, ne) in &normals {
        emit_div(zero_f, zero_e, nf, ne);
        count += 1;
    }

    // Normal / Infinity → Zero
    for &(nf, ne) in &normals {
        emit_div(nf, ne, inf_f, inf_e);
        count += 1;
    }

    // Undefined / anything → passthrough
    for &uf in &undef_fracs {
        emit_div(uf, ambig, normals[0].0, normals[0].1);
        count += 1;
        emit_div(uf, ambig, zero_f, zero_e);
        count += 1;
        emit_div(uf, ambig, inf_f, inf_e);
        count += 1;
    }

    // Anything / Undefined → passthrough
    for &uf in &undef_fracs {
        emit_div(normals[0].0, normals[0].1, uf, ambig);
        count += 1;
        emit_div(zero_f, zero_e, uf, ambig);
        count += 1;
        emit_div(inf_f, inf_e, uf, ambig);
        count += 1;
    }

    // Exploded / Exploded
    for &ef1 in &exploded_fracs {
        for &ef2 in &exploded_fracs {
            emit_div(ef1, ambig, ef2, ambig);
            count += 1;
        }
    }

    // Vanished / Vanished
    for &vf1 in &vanished_fracs {
        for &vf2 in &vanished_fracs {
            emit_div(vf1, ambig, vf2, ambig);
            count += 1;
        }
    }

    // Exploded / Normal → NR computes (exploded is N1-normalized)
    for &(nf, ne) in &normals {
        for &ef in &exploded_fracs {
            emit_div(ef, ambig, nf, ne);
            count += 1; // exploded÷normal → N1
            emit_div(nf, ne, ef, ambig);
            count += 1; // normal÷exploded → N2
        }
    }

    // Vanished / Normal → GENERAL (vanished is N2, can't feed to NR LUT)
    // Vanished / Exploded → GENERAL
    // Exploded / Vanished → GENERAL
    let general_frac: i16 = 0xFE00u16 as i16; // GENERAL prefix 0xFE, sa-aligned
    for &vf in &vanished_fracs {
        for &(nf, ne) in &normals {
            emit_line_div(vf, ambig, nf, ne, general_frac, ambig);
            count += 1;
            emit_line_div(nf, ne, vf, ambig, general_frac, ambig);
            count += 1;
        }
        for &ef in &exploded_fracs {
            emit_line_div(vf, ambig, ef, ambig, general_frac, ambig);
            count += 1;
            emit_line_div(ef, ambig, vf, ambig, general_frac, ambig);
            count += 1;
        }
    }

    // === SQRT ===

    // Zero → Zero
    emit_sqrt(zero_f, zero_e);
    count += 1;

    // Negative normal → SQRT_NEGATIVE
    emit_sqrt(-0x4000, 0);
    count += 1;
    emit_sqrt(-0x6000, 5);
    count += 1;
    emit_sqrt(-0x7FFF, -3);
    count += 1;

    // Positive normal (sanity)
    emit_sqrt(0x4000, 0);
    count += 1;
    emit_sqrt(0x6000, 4);
    count += 1;

    // Infinity → SQRT_EXPLODED
    emit_sqrt(inf_f, inf_e);
    count += 1;

    // Exploded → SQRT_EXPLODED
    for &ef in &exploded_fracs {
        emit_sqrt(ef, ambig);
        count += 1;
    }

    // Vanished → SQRT_VANISHED
    for &vf in &vanished_fracs {
        emit_sqrt(vf, ambig);
        count += 1;
    }

    // Undefined → passthrough
    for &uf in &undef_fracs {
        emit_sqrt(uf, ambig);
        count += 1;
    }

    // N0 with AMBIG (0xFFFF) → passthrough
    emit_sqrt(-1i16, ambig);
    count += 1;

    eprintln!("Generated {} edge case vectors", count);
}
