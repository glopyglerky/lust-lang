// Release semantics: MIN / -1 still panics (the division overflow check is not an
// overflow-checks check), as in a debug build.
use std::hint::black_box as bb;

fn main() {
    println!("{}", bb(i32::MIN).wrapping_div(bb(-1)));
    println!("{}", bb(7i32) % bb(-1));
    let q = bb(i64::MIN) / bb(-1);
    println!("not reached {q}");
}
