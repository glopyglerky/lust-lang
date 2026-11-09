// expect: L0631
// rustc E0492: a promoted borrow of an interior-mutable temporary.
use std::sync::atomic::{AtomicU8, Ordering};
static S: &AtomicU8 = &AtomicU8::new(3);
fn main() { S.store(9, Ordering::Relaxed); println!("{}", S.load(Ordering::Relaxed)); }
