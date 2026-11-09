// panic: overflow in Duration::new
use std::time::Duration;
fn main() { let s = u64::MAX; let d = Duration::new(s, 1_000_000_000); println!("{}", d.as_secs()); }
