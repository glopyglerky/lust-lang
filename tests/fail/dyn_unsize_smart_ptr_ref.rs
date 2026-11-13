// expect: L0612
// rustc E0277: a `&dyn Tr` target commits to unsizing, so `&Rc<S>` needs
// `Rc<S>: Tr`; the `&Rc<S>` to `&S` deref coercion is not tried.
use std::rc::Rc;
trait Tr { fn read(&self) -> u32; }
struct S { n: u32 }
impl Tr for S { fn read(&self) -> u32 { self.n } }
fn main() {
    let r = Rc::new(S { n: 5 });
    let d: &dyn Tr = &r;
    println!("{}", d.read());
}
