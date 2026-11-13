// Cell and RefCell stores through named lifetimes that Rust accepts.
use std::cell::{Cell, RefCell};
fn put<'a>(c: &Cell<&'a i32>, x: &'a i32) {
    c.set(x);
}
fn push<'a>(r: &RefCell<Vec<&'a str>>, s: &'a str) {
    r.borrow_mut().push(s);
}
fn first<'a>(r: &'a RefCell<Vec<i32>>) -> i32 {
    r.borrow()[0]
}
fn main() {
    let a = 1;
    let b = 2;
    let c = Cell::new(&a);
    put(&c, &b);
    let owned = String::from("x");
    let r = RefCell::new(vec!["s"]);
    push(&r, &owned);
    let n = RefCell::new(vec![9]);
    println!("{} {:?} {}", c.get(), r.borrow(), first(&n));
}
