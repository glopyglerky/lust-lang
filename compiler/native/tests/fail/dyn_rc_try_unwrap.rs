// expect: L0110
// rustc E0277: `Rc::try_unwrap` needs a sized value.
use std::rc::Rc;
trait A { fn a(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
fn main() { let r: Rc<dyn A> = Rc::new(S); let v = Rc::try_unwrap(r); let _ = v; }
