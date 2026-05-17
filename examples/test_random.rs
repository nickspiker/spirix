use spirix::*;

fn main() {
    let mut in_range = 0;
    let mut vanished = 0;
    let mut max_seen = ScalarF3E3::ZERO;
    let mut min_seen = ScalarF3E3::ZERO;
    let mut binade_hist = [0u32; 9]; // logical k = -1, -2, ..., -8

    let n = 100_000;
    for _ in 0..n {
        let r: ScalarF3E3 = ScalarF3E3::random();

        // Range is [-1, +1): -1 is reachable (fraction=0 at largest stored exp), +1 is not.
        if r >= 1 {
            panic!("r >= 1 found: {:?}", r);
        }
        if r < -1 {
            panic!("r < -1 found: {:?}", r);
        }
        if r < min_seen {
            min_seen = r;
        }
        if r > max_seen {
            max_seen = r;
        }

        if r.vanished() {
            vanished += 1;
        } else if !r.is_zero() {
            in_range += 1;
            // crude: log2(magnitude) bucket
            let mag = r.magnitude().to_f64();
            if mag > 0.0 {
                let k = (-mag.log2()).floor() as usize;
                if k < binade_hist.len() {
                    binade_hist[k] += 1;
                }
            }
        }
    }

    println!("n = {}", n);
    println!("in normal range: {} ({:.2}%)", in_range, 100.0 * in_range as f64 / n as f64);
    println!("vanished: {} ({:.4}%)", vanished, 100.0 * vanished as f64 / n as f64);
    println!("min = {}, max = {}", min_seen.to_f64(), max_seen.to_f64());
    println!("\nBinade histogram (logical k = -1, -2, ...):");
    for (k, count) in binade_hist.iter().enumerate() {
        let expected = n as f64 * (0.5f64).powi(k as i32 + 1);
        println!(
            "  k=-{}: |x|∈[2^{}, 2^{}): {:6} (expected ~{:.0}, {:.1}%)",
            k + 1,
            -(k as i32 + 1),
            -(k as i32),
            count,
            expected,
            100.0 * *count as f64 / n as f64
        );
    }
}
