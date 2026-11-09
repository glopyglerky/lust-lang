// expect: L0612
// rustc E0277: a written type S<Rc<u8>> in a signature must satisfy S's T: Send bound.
use std::rc::Rc;
struct S<T: Send>(T);
fn f(_s: &S<Rc<u8>>) {}
fn main() { println!("x"); }
