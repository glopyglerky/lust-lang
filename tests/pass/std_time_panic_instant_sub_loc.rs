// panic: std_time_panic_instant_sub_loc.rs:7:13:
// `Instant - Duration` overflow reports the user's `-` site
// (fz2-platform).
use std::time::{Duration, Instant};
fn main() {
    let t = Instant::now();
    let u = t - Duration::MAX;
    println!("{}", u > t);
}
