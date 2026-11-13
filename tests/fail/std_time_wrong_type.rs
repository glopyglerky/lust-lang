// expect: L0113
// rustc E0308: `Duration * Duration` has no impl.
use std::time::Duration;
fn main() {
    let d = Duration::from_secs(2) * Duration::from_secs(3);
    println!("{:?}", d);
}
