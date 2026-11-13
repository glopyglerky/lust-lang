// expect: L0104
// rustc E0658: `Duration::from_days` is unstable (duration_constructors).
use std::time::Duration;
fn main() {
    println!("{:?}", Duration::from_days(2));
}
