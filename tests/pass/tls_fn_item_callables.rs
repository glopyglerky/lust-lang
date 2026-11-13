// User fn items and fn pointers as `with`, `with_borrow`, and
// `with_borrow_mut` callables.
use std::cell::{Cell, RefCell};
thread_local! {
    static N: Cell<u8> = const { Cell::new(1) };
    static V: RefCell<Vec<u32>> = RefCell::new(vec![1, 2]);
}
fn show(c: &Cell<u8>) -> u8 {
    c.get() + 1
}
fn len(v: &Vec<u32>) -> usize {
    v.len()
}
fn push(v: &mut Vec<u32>) {
    v.push(9)
}
fn main() {
    println!("{}", N.with(show));
    println!("{}", V.with_borrow(len));
    V.with_borrow_mut(push);
    println!("{}", V.with_borrow(len));
    let f: fn(&Cell<u8>) -> u8 = show;
    println!("{}", N.with(f));
}
