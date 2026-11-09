// expect: L0542
use std::sync::atomic::{AtomicUsize, Ordering};
fn main() { let a = AtomicUsize::new(1); println!("{}", a.load(Ordering::Release)); }
