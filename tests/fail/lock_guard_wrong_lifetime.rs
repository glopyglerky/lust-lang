// expect: L0733
use std::sync::{Mutex, MutexGuard};
fn get<'a, 'b>(_m: &'a Mutex<i32>, n: &'b Mutex<i32>) -> MutexGuard<'a, i32> { n.lock().unwrap() }
fn main() { let m = Mutex::new(1); let n = Mutex::new(5); println!("{}", *get(&m, &n)); }
