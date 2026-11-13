// expect: L0110
// rustc E0308: an `Rc<S>` does not coerce to an `Arc<dyn T>`.
use std::sync::Arc; use std::rc::Rc;
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn main() { let a: Arc<dyn T> = Rc::new(S); println!("{}", a.f()); }
