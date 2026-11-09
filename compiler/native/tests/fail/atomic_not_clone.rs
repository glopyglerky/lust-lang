// expect: L0104
use std::sync::atomic::AtomicU16;
fn main() { let a = AtomicU16::new(5); let b = a.clone(); println!("{:?}", b); }
