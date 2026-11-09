// expect: L0305
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(String::from("a"));
    let s = *c.borrow();
    println!("{}", s);
}
