use spirix::*;

fn debug_round(val: ScalarF5E3) -> ScalarF5E3 {
    println!("=== Debugging round({}) ===", val.to_f64());

    // Handle non-normal values first
    if !val.is_normal() {
        if val.vanished() {
            println!("Vanished -> ZERO");
            return ScalarF5E3::ZERO;
        }
        println!("Non-normal -> self");
        return val;
    }

    // Use math approach for banker's rounding
    let floored = val.floor();
    let frac = val - floored;

    println!("floored = {}, frac = {}", floored.to_f64(), frac.to_f64());

    // Check if fractional part is exactly 0.5
    let diff = frac - ScalarF5E3::HALF;
    let is_exactly_half = diff.magnitude() < ScalarF5E3::MIN_POS;

    println!("diff = {}, is_exactly_half = {}", diff.to_f64(), is_exactly_half);

    if is_exactly_half {
        // Banker's rounding: round to even integer
        let two = ScalarF5E3::ONE + ScalarF5E3::ONE;
        let is_even = (floored % two).magnitude() < ScalarF5E3::MIN_POS;

        println!("is_even = {}", is_even);

        if is_even {
            // Even integer: keep it (round down)
            println!("Even -> return floored = {}", floored.to_f64());
            return floored;
        } else {
            // Odd integer: round to next even (round away from zero)
            if val.is_positive() {
                let result = floored + ScalarF5E3::ONE;
                println!("Odd, positive -> return {} + 1 = {}", floored.to_f64(), result.to_f64());
                return result;
            } else {
                let result = floored - ScalarF5E3::ONE;
                println!("Odd, negative -> return {} - 1 = {}", floored.to_f64(), result.to_f64());
                return result;
            }
        }
    } else if frac > ScalarF5E3::HALF {
        // Round up
        if val.is_positive() {
            let result = floored + ScalarF5E3::ONE;
            println!("Frac > 0.5, positive -> return {} + 1 = {}", floored.to_f64(), result.to_f64());
            return result;
        } else {
            let result = floored - ScalarF5E3::ONE;
            println!("Frac > 0.5, negative -> return {} - 1 = {}", floored.to_f64(), result.to_f64());
            return result;
        }
    } else {
        // Round down
        println!("Frac < 0.5 -> return floored = {}", floored.to_f64());
        return floored;
    }
}

fn main() {
    let val = ScalarF5E3::from(2.5f32);
    let result_debug = debug_round(val);
    let result_actual = val.round();

    println!("Debug result: {}", result_debug.to_f64());
    println!("Actual result: {}", result_actual.to_f64());
    println!("Match? {}", result_debug == result_actual);
}