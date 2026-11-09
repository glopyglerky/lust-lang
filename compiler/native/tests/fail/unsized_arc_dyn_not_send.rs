// expect: L0612
// rustc E0277: `Rc<u8>` is not `Send`, so `S` cannot become a `dyn T + Send`.
use std::sync::Arc; use std::rc::Rc;
trait T { fn f(&self) -> u8; }
struct S(Rc<u8>);
impl T for S { fn f(&self) -> u8 { *self.0 } }
fn main() { let a: Arc<dyn T + Send> = Arc::new(S(Rc::new(1))); println!("{}", a.f()); }
