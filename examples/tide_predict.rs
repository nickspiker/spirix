// Tide-prediction numeric proof for the nRF52840 project.
//
// Question: can a Spirix Scalar, at some tunable (fraction, exponent) width,
// compute a harmonic tide sum `Σ aᵢ·cos(ωᵢ·t − φᵢ)` accurately enough that the
// height error stays below the clock-margin floor (~0.0005 ft), while feeding
// the RAW large phase argument ωᵢ·t (billions of radians) into `.cos()`?
//
// The known risk is argument reduction: cos() reduces mod τ via `.frac()`, so a
// huge integer quotient eats fraction bits before any describe the angle. This
// program computes the same sum in f64 (reference) and in several Spirix widths,
// then reports max error vs f64 — showing exactly where each width breaks.
//
// Run: cargo run --release --example tide_predict

use spirix::*;

// Bremerton 9445958 constituents: (speed °/hr, amplitude ft, phase_GMT °).
// From NOAA harcon.json. Node factors / equilibrium args omitted — this proof
// isolates the NUMERIC question (does Spirix hold precision at these arg
// magnitudes), comparing Spirix against f64 computing the identical formula.
const CONSTITUENTS: &[(f64, f64, f64)] = &[
    (28.9841040, 3.60, 17.0), (30.0000000, 0.89, 45.1), (28.4397300, 0.70, 349.0),
    (15.0410690, 2.73, 280.4), (57.9682100, 0.08, 243.9), (13.9430350, 1.50, 257.8),
    (86.9523200, 0.06, 3.0),   (44.0251730, 0.09, 120.4), (60.0000000, 0.01, 289.5),
    (57.4238320, 0.04, 213.7), (28.5125830, 0.15, 358.1), (27.9682080, 0.08, 231.9),
    (27.8953550, 0.09, 322.3), (16.1391010, 0.10, 333.4), (29.4556260, 0.06, 45.4),
    (15.0000000, 0.09, 17.5),  (14.4966940, 0.05, 323.0), (15.5854435, 0.15, 319.5),
    (0.5443747, 0.00, 0.0),    (0.0821373, 0.11, 231.1),  (0.0410686, 0.25, 292.9),
    (1.0980331, 0.07, 140.5),  (13.4715150, 0.05, 272.1), (13.3986610, 0.24, 251.2),
    (29.9589330, 0.06, 49.1),  (30.0410670, 0.01, 354.8), (12.8542860, 0.04, 230.5),
    (14.9589310, 0.83, 281.3), (31.0158960, 0.03, 270.2), (43.4761600, 0.02, 207.2),
    (29.5284790, 0.12, 63.4),  (42.9271400, 0.06, 80.3),  (30.0821380, 0.26, 39.0),
    (115.9364200, 0.01, 224.8), (58.9841040, 0.05, 267.2),
];
const MSL_FT: f64 = 18.20;

// Reference height in f64. `t_hours` = hours since the harmonic epoch.
fn height_f64(t_hours: f64) -> f64 {
    let mut h = MSL_FT;
    for &(speed_deg_hr, amp, phase_deg) in CONSTITUENTS {
        let arg = (speed_deg_hr * t_hours - phase_deg).to_radians();
        h += amp * arg.cos();
    }
    h
}

// Same sum in a Spirix Scalar type S. The phase argument ωt is built and reduced
// entirely in S — the stress test. `to_f64()`/`from` bridge only at the edges.
macro_rules! height_spirix {
    ($S:ty, $t_hours:expr) => {{
        type S = $S;
        let deg2rad = S::from(std::f64::consts::PI / 180.0);
        let t = S::from($t_hours);
        let mut h = S::from(MSL_FT);
        for &(speed_deg_hr, amp, phase_deg) in CONSTITUENTS {
            let arg = (S::from(speed_deg_hr) * t - S::from(phase_deg)) * deg2rad;
            h = h + S::from(amp) * arg.cos();
        }
        h.to_f64()
    }};
}

fn main() {
    // EXPONENT RANGE PROBE (E3 = i8 exponent).
    // Push t_hours outward across many years and watch where F5E3 diverges from
    // f64 — that reveals when the i8 exponent (range ~2^127) runs out for the
    // magnitudes flowing through the harmonic sum (dominated by omega*t).
    println!("Exponent-range probe: F5E3 (i8 exp) vs f64, growing t
");
    println!("  {:>14}  {:>10}  {:>14}  {}", "t_hours", "years", "max omega*t", "F5E3 max_err (ft)");
    let year_hours = 8766.0_f64;
    for &years in &[1.0_f64, 10.0, 100.0, 1_000.0, 1e4, 1e5, 1e6, 1e9, 1e12, 1e15, 1e18, 1e30, 1e36] {
        let base = years * year_hours;
        let max_arg = (115.9364_f64 * (base + 24.0)).to_radians();
        let mut max_err = 0.0_f64;
        for step in 0..=24 {
            let t = base + step as f64;
            let r = height_f64(t);
            let g = height_spirix!(ScalarF5E3, t);
            let e = (g - r).abs();
            if e.is_finite() { max_err = max_err.max(e); } else { max_err = f64::INFINITY; }
        }
        println!("  {:>14.3e}  {:>10.0e}  {:>14.3e}  {:e}", base, years, max_arg, max_err);
    }
    // And where does the i8 exponent itself saturate? Largest magnitude F5E3 holds.
    println!("
Raw magnitude probe (F5E3): largest 2^k it represents before exploding:");
    for k in [100i32, 120, 125, 126, 127, 128, 130, 200] {
        let v = 2f64.powi(k);
        let s = ScalarF5E3::from(v);
        let back = s.to_f64();
        let ok = back.is_finite() && (back/v - 1.0).abs() < 0.01;
        println!("  2^{:<4} = {:.3e}  ->  F5E3 back = {:.3e}  {}", k, v, back, if ok {"ok"} else {"OVERFLOW/exploded"});
    }
}
