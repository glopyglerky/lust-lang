// expect: L0542
use std::sync::atomic::{fence, Ordering};
fn main() { fence(Ordering::Relaxed); }
