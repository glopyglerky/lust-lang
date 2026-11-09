// expect: L0113
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel::<u8>(); for x in &mut rx { let _ = x; } drop(tx); }
