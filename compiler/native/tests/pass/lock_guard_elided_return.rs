use std::sync::{Mutex, MutexGuard};
fn get(m: &Mutex<i32>) -> MutexGuard<i32> { m.lock().unwrap() }
fn main() { let m = Mutex::new(3); *get(&m) += 1; println!("{:?}", m); }
