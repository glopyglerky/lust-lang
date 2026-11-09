// expect: L0104
use std::sync::atomic::{AtomicBool, Ordering};
fn main() { let b = AtomicBool::new(true); println!("{}", b.fetch_add(true, Ordering::SeqCst)); }
