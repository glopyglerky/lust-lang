// panic: std_time_panic_system_sub_loc.rs:7:13:
// `SystemTime - Duration` overflow reports the user's `-` site
// (fz2-platform).
use std::time::{Duration, SystemTime};
fn main() {
    let t = std::time::UNIX_EPOCH;
    let u = t - Duration::MAX;
    println!("{}", u > t);
}
