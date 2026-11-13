// An overflow check in a #[track_caller] fn reports the caller's location (W-CHECK-G3).
// panic: track_caller_overflow.rs:5:55:
#[track_caller]
fn add(a: u8, b: u8) -> u8 { a + b }
fn main() { println!("{}", add(1, 2)); println!("{}", add(200, 100)); }
