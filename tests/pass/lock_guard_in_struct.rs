use std::sync::{Mutex, MutexGuard};
struct H<'a> { g: MutexGuard<'a, Vec<u8>> }
fn main() { let m = Mutex::new(vec![1u8]); let mut h = H { g: m.lock().unwrap() }; h.g.push(2); drop(h); println!("{:?}", m.lock().unwrap()); }
