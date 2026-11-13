// expect: L0631
// rustc E0015: `Duration::from_secs_f64` is not a const fn.
use std::time::Duration;
const D: Duration = Duration::from_secs_f64(1.5);
fn main() {
    println!("{:?}", D);
}
