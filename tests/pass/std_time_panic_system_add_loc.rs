// panic: std_time_panic_system_add_loc.rs:7:13:
// `SystemTime + Duration` overflow reports the user's `+` site
// (fz2-platform).
use std::time::{Duration, SystemTime};
fn main() {
    let t = SystemTime::now();
    let u = t + Duration::MAX;
    println!("{}", u > t);
}
