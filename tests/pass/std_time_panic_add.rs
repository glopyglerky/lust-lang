// panic: overflow when adding durations
// `Duration + Duration` panics with std's message on overflow
// (library/core/src/time.rs, W-STD-C).
use std::time::Duration;
fn main() {
    let d = Duration::MAX - Duration::from_secs(1);
    println!("{:?}", d.checked_add(Duration::from_secs(2)));
    println!("{:?}", d + Duration::from_millis(500));
    let e = d + Duration::from_secs(2);
    println!("unreachable {:?}", e);
}
