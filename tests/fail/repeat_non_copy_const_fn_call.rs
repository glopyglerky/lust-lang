// expect: L0110
// rustc E0277: a const fn call makes one value; repeating it needs Copy.
use std::sync::atomic::{AtomicUsize, Ordering};
static Y: [AtomicUsize; 3] = [AtomicUsize::new(0); 3];
fn main() { Y[1].fetch_add(5, Ordering::SeqCst); println!("{:?}", Y); }
