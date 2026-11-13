// expect: L0308
use std::rc::Rc;
fn main() {
    let mut v = Rc::new(vec![1]);
    v.push(2);
    println!("{:?}", v);
}
