// expect: L0612
// rustc E0277: `Arc<dyn T>` is not `Send` (`dyn T` is neither `Send` nor `Sync`).
use std::sync::Arc; use std::thread;
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn mk() -> Arc<dyn T> { Arc::new(S) }
fn main() { let a = mk(); let h = thread::spawn(move || a.f()); println!("{}", h.join().unwrap()); }
