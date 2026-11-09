// expect: L0306
use std::sync::{Mutex, MutexGuard};
fn get<'a>(_m: &'a Mutex<i32>, n: &'a Mutex<i32>) -> MutexGuard<'a, i32> { n.lock().unwrap() }
fn main() { let m = Mutex::new(1); let g; { let n = Mutex::new(5); g = get(&m, &n); } println!("{}", *g); }
