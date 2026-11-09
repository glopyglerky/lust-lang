// panic: RefCell already borrowed
use std::cell::RefCell;
fn main() {
    let r = RefCell::new(String::from("a"));
    let g = r.borrow();
    let s = r.take();
    println!("{} {}", g, s);
}
