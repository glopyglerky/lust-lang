// expect: L0612
// rustc E0277: `Rc<u8>` is not `Send`, so it cannot become a `Box<dyn Send>`.
use std::rc::Rc;
fn main() {
    let b: Box<dyn Send> = Box::new(Rc::new(1u8));
    drop(b);
}
