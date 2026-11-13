// expect: L0631
use std::sync::atomic::AtomicU8;
const fn make() -> AtomicU8 { AtomicU8::from(3) }
fn main() { println!("{:?}", make()); }
