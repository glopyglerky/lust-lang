// expect: L0102
// `as _` binds no name.
use std::rc::Rc as _;
fn main() {
    let b = Rc::new(2);
    println!("{}", b);
}
