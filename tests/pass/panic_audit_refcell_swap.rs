// `RefCell::swap` is not track_caller: a borrowed `other` panics at its
// `borrow_mut` in core's cell.rs.
// panic: RefCell already borrowed
use std::cell::RefCell;
fn main() {
    let a = RefCell::new(String::from("a"));
    let b = RefCell::new(String::from("b"));
    a.swap(&b);
    println!("{} {}", a.borrow(), b.borrow());
    let g = b.borrow();
    a.swap(&b);
    println!("{}", g);
}
