// expect: L0307
use std::sync::{Mutex, MutexGuard};
fn get<'a>() -> MutexGuard<'a, i32> { let m = Mutex::new(1); m.lock().unwrap() }
fn main() { println!("{}", *get()); }
