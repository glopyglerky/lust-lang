// expect: L0612
// rustc E0277: `Rc<[u16]>` has no `From<Vec<u8>>`.
use std::rc::Rc;
fn main() { let v = vec![1u8]; let a: Rc<[u16]> = Rc::from(v); println!("{:?}", a); }
