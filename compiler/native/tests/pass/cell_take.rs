// Cell::take and RefCell::take leave T::default() behind.
use std::cell::{Cell, RefCell};
#[derive(Default, Debug)]
struct S {
    n: u8,
    s: String,
}
fn main() {
    let c = Cell::new(5u32);
    let old = c.take();
    println!("{} {}", old, c.get());
    let r = RefCell::new(vec![1, 2]);
    let v = r.take();
    println!("{:?} {:?}", v, r.borrow());
    let rs = RefCell::new(S { n: 3, s: String::from("x") });
    let t = rs.take();
    println!("{:?} {:?}", t, rs.borrow());
    let o: Cell<Option<u8>> = Cell::new(Some(4));
    println!("{:?} {:?}", o.take(), o.get());
}
