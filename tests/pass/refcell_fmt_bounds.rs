// Debug and Display bounds on RefCell, Cell, and the guards Ref and RefMut (W-STD-B): std's impls through T.
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Display};
use std::rc::Rc;

fn show<T: Debug>(t: T) {
    println!("{:?}", t);
}

fn text<T: Display>(t: T) {
    println!("[{}]", t);
}

fn main() {
    let c = RefCell::new(7);
    show(RefCell::new(1));
    show(Cell::new(2));
    show(c.borrow());
    show(c.borrow_mut());
    let s = RefCell::new(String::from("hi"));
    text(s.borrow());
    text(s.borrow_mut());
    println!("{} {:?}", s.borrow(), s);
    show(Rc::new(RefCell::new(vec![1, 2])));
    let held = c.borrow();
    println!("{:?} {}", c.try_borrow_mut().unwrap_err(), *held);
}
