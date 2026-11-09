// expect: L0612
// rustc E0277: Rc<T>: Default needs T: Default.
use std::rc::Rc;
struct N;
fn main() { let _a: Rc<N> = Rc::default(); }
