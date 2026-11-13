// RefCell::default, Cell::default, and Default::default for both (W-STD-B), also as struct fields.
use std::cell::{Cell, RefCell};

#[derive(Debug)]
struct S {
    c: RefCell<Vec<u8>>,
    n: Cell<u32>,
}

fn main() {
    let s = S { c: RefCell::default(), n: Cell::default() };
    s.c.borrow_mut().push(1);
    let r: RefCell<String> = RefCell::default();
    println!("{:?} {:?}", s, r);
}
