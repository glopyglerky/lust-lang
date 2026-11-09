// panic: std_time_panic_duration_div_assign_loc.rs:9:5:
// `Duration /= 0` reports the user's `/=` site (fz2-platform).
use std::time::Duration;
fn zero() -> u32 {
    0
}
fn main() {
    let mut d = Duration::from_secs(9);
    d /= zero();
    println!("{:?}", d);
}
