// expect: L0110
// rustc E0308: `Rc::clone(&&a)` derefs to the `Rc` and returns an
// `Rc<i32>`, never a clone of the reference.
use std::rc::Rc;
fn main() {
    let a = Rc::new(1);
    let w: &Rc<i32> = Rc::clone(&&a);
    println!("{:?}", w);
}
