// Default for RefCell and Cell through #[derive(Default)] and with a turbofish (W-STD-B).
use std::cell::{Cell, RefCell};

#[derive(Default, Debug)]
struct S {
    a: RefCell<u8>,
    b: Cell<bool>,
}

fn main() {
    let x = RefCell::<String>::default();
    x.borrow_mut().push('a');
    println!("{:?}", x.borrow().len());
    let s = S::default();
    s.b.set(true);
    *s.a.borrow_mut() += 2;
    println!("{:?}", s);
    let c = Cell::<u32>::default();
    c.set(c.get() + 5);
    println!("{}", c.get());
}
