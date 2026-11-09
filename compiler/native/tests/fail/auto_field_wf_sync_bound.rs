// expect: L0612
// rustc E0277: the field type S<Rc<u8>> must satisfy S's T: Sync bound.
use std::rc::Rc;
struct S<T: Sync>(T);
struct U { s: S<Rc<u8>> }
fn main() { println!("x"); }
