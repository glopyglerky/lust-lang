// expect: L0630
// rustc E0015: `Duration + Duration` is not const in stable Rust.
use std::time::Duration;
const D: Duration = Duration::from_secs(1) + Duration::from_secs(2);
fn main() {
    println!("{:?}", D);
}
