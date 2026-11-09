// panic: divide by zero error when dividing duration by scalar
// `Duration / 0` panics with std's message (W-STD-C).
use std::time::Duration;
fn zero() -> u32 {
    0
}
fn main() {
    let d = Duration::from_secs(9);
    println!("{:?}", d / 2);
    println!("{:?}", d / zero());
}
