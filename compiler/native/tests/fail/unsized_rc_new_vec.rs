// expect: L0110
// rustc E0308: `Rc::new(vec![1])` is an `Rc<Vec<i32>>`, which does not coerce to `Rc<[i32]>`.
use std::rc::Rc;
fn main() { let a: Rc<[i32]> = Rc::new(vec![1]); println!("{:?}", a); }
