// expect: L0615
// rustc E0119: alloc has `impl<T> From<Box<T>> for Rc<T>`.
use std::rc::Rc;
struct Local;
impl From<Box<Local>> for Rc<Local> { fn from(_: Box<Local>) -> Self { Rc::new(Local) } }
fn main() { println!("ok"); }
