// expect: L0102
use std::rc::Rc;
fn main() {
    let w = Weak::<i32>::new();
    println!("{:?}", w.upgrade());
    let r = Rc::new(1);
    println!("{}", r);
}
