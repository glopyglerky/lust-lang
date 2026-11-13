// expect: L0104
use std::sync::Barrier;
fn main() { let b = Barrier::new(1); let b2 = b.clone(); }
