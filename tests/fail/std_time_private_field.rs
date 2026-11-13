// expect: L0510
// rustc E0616: `Duration`'s fields are private.
use std::time::Duration;
fn main() {
    let d = Duration::from_secs(1);
    println!("{}", d.secs);
}
