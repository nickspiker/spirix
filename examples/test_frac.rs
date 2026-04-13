use spirix::ScalarF7E7;

fn main() {
    let mut pass = 0u64;
    let mut fail = 0u64;

    for _ in 0..1_000_000 {
        let a = ScalarF7E7::random();
        let frac_new = a.frac();
        let frac_old = a - a.floor();

        if frac_new == frac_old {
            pass += 1;
        } else {
            if fail < 20 {
                println!(
                    "MISMATCH: a={a}  frac_new={frac_new}  frac_old={frac_old}  floor={}",
                    a.floor()
                );
            }
            fail += 1;
        }
    }

    println!("\n{pass}/{} pass, {fail} fail", pass + fail);
    if fail == 0 {
        println!("ALL PASS");
    }
}
