// panic: cannot convert float seconds to Duration: value is negative
// `Duration::from_secs_f64` of a negative value panics with the
// `TryFromFloatSecsError` text (W-STD-C).
use std::time::Duration;
fn main() {
    println!("{:?}", Duration::from_secs_f64(0.25));
    println!("{:?}", Duration::from_secs_f64(-0.25));
}
