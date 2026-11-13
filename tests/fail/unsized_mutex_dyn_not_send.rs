// expect: L0612
// rustc E0277: `dyn T` is not `Send`, so `Arc<Mutex<dyn T>>` cannot move to another thread.
use std::sync::{Arc, Mutex};
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn mk() -> Arc<Mutex<dyn T>> { Arc::new(Mutex::new(S)) }
fn main() { let a = mk(); std::thread::spawn(move || a.lock().unwrap().f()); }
