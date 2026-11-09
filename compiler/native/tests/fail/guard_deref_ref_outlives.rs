// expect: L0733
// `MutexGuard::deref` ties the result to the borrow of the guard, not to
// the guard's own lifetime.
use std::sync::{Mutex, MutexGuard};
fn get<'a, 'b>(r: &'a MutexGuard<'b, Vec<u8>>) -> &'b Vec<u8> {
    r
}
fn main() {
    let c = Mutex::new(vec![1u8]);
    let g = c.lock().unwrap();
    println!("{}", get(&g).len());
}
