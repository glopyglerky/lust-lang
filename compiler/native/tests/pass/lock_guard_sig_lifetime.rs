use std::sync::{Mutex, MutexGuard};
fn get<'a>(m: &'a Mutex<i32>, _n: &i32) -> MutexGuard<'a, i32> { m.lock().unwrap() }
fn main() { let m = Mutex::new(1); let g; { let n = 5; g = get(&m, &n); } println!("{}", *g); }
