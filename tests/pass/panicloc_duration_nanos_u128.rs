// Duration::from_nanos_u128 is `#[track_caller]` in std 1.99: its overflow
// panic names the caller (tf-panicloc).
use std::time::Duration;
fn main() {
    let n = std::env::args().count() as u128;
    println!("{:?}", Duration::from_nanos_u128(1_500_000_000 * n));
    println!("{:?}", Duration::from_nanos_u128(u128::MAX - n));
}
