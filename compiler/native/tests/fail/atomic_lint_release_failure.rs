// expect: L0542
use std::sync::atomic::{AtomicBool, Ordering};
fn main() { let a = AtomicBool::new(true); let _ = a.compare_exchange_weak(true, false, Ordering::SeqCst, Ordering::Release); }
