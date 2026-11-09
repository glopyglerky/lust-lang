// panic: std_time_panic_instant_add_loc.rs:7:13:
// `Instant + Duration` overflow reports the user's `+` site, as
// rustc's #[track_caller] operator does (fz2-platform).
use std::time::{Duration, Instant};
fn main() {
    let t = Instant::now();
    let u = t + Duration::MAX;
    println!("{}", u > t);
}
