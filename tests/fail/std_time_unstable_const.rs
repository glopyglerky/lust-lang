// expect: L0104
// rustc E0658: `Duration::SECOND` is unstable (duration_constants).
use std::time::Duration;
fn main() {
    println!("{:?}", Duration::SECOND);
}
