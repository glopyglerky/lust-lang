// expect: L0612
// rustc E0277: `Rc<dyn B>: From<Box<dyn A>>` does not hold; the pointee must
// be the same type.
use std::rc::Rc;
trait A {}
trait B {}
struct S;
impl A for S {}
impl B for S {}
fn main() {
    let b: Box<dyn A> = Box::new(S);
    let _r: Rc<dyn B> = Rc::from(b);
}
