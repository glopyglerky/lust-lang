// expect: L0110
// rustc E0599: `Mutex::into_inner` moves the value out, so it needs `T: Sized`.
use std::sync::Mutex;
trait T: Send { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn main() { let a: Box<Mutex<dyn T>> = Box::new(Mutex::new(S)); let v = a.into_inner(); }
