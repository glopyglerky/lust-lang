// Debug of the std cell types, Weak, ExitCode, and Box<[T]> through the
// Formatter builders: pretty, padded, and a borrowed RefCell.
use std::cell::{Cell, RefCell};
use std::process::ExitCode;
use std::rc::{Rc, Weak};
#[derive(Debug, Clone, Copy, PartialEq)]
struct P { x: i32, y: i32 }
fn main() {
    let c = Cell::new(P { x: 1, y: -2 });
    let r = RefCell::new(vec![P { x: 3, y: 4 }]);
    let b: Box<[P]> = vec![P { x: 5, y: 6 }, P { x: 7, y: 8 }].into_boxed_slice();
    let e: Box<[u8]> = Vec::new().into_boxed_slice();
    let rc = Rc::new(RefCell::new(9u8));
    let w: Weak<RefCell<u8>> = Rc::downgrade(&rc);
    println!("{:#?}", c);
    println!("{:#?}", r);
    println!("{:#?} {:?} {:#?}", b, e, e);
    println!("{:>12?}|{:<8?}|", w, ExitCode::SUCCESS);
    println!("{:#?}", ExitCode::from(3));
    let g = r.borrow_mut();
    println!("{:?} {:#?}", r, r);
    drop(g);
    println!("{:5?}|{:<5?}|{:+?}", Cell::new(1), RefCell::new(2), Cell::new(3));
    println!("{:?}", (Some(Cell::new('x')), [RefCell::new("s")]));
}
