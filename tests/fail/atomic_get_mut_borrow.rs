// expect: L0303
use std::sync::atomic::{AtomicUsize, Ordering};
fn main() { let mut a = AtomicUsize::new(1); let r = a.get_mut(); a.store(3, Ordering::SeqCst); *r += 1; println!("{:?}", a); }
