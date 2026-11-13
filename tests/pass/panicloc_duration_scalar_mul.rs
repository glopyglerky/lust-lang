// `u32 * Duration` overflows inside core's `Mul<u32> for Duration`, which
// is not `#[track_caller]`: rustc names core/src/time.rs (tf-panicloc).
use std::time::Duration;
fn main() {
    let n = std::env::args().count() as u32;
    println!("{:?}", n * Duration::from_secs(3));
    let mut d = Duration::MAX;
    d *= n + 1;
    println!("{:?}", d);
}
