// panic: overflow when adding duration to instant
// `Instant + Duration` past the clock range panics (W-STD-C).
use std::time::{Duration, Instant};
fn main() {
    let t = Instant::now();
    println!("{}", t.checked_add(Duration::MAX).is_none());
    let u = t + Duration::MAX;
    println!("unreachable {}", u > t);
}
