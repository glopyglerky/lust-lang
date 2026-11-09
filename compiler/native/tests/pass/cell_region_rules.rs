// Borrows through Cell and RefCell that Rust accepts: guards end at
// their drop, a value read out of a cell is not tied to the cell, and
// Rc clones share one cell.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
fn main() {
    let c = RefCell::new(1);
    {
        let r = c.borrow();
        println!("{}", r);
    }
    let g = c.borrow_mut();
    drop(g);
    println!("{}", c.into_inner());

    let x = 5;
    let cx = Cell::new(&x);
    let y = 6;
    cx.set(&y);
    println!("{}", cx.get());

    let a = 1;
    let ca = Cell::new(&a);
    let got = ca.get();
    drop(ca);
    println!("{}", got);

    let cs = RefCell::new("x");
    *cs.borrow_mut() = "y";
    let s: &str = *cs.borrow();
    let owned = cs.into_inner();
    println!("{} {}", s, owned);

    let r = Rc::new(RefCell::new(vec![&x]));
    let r2 = Rc::clone(&r);
    r2.borrow_mut().push(&y);
    let total: i32 = r.borrow().iter().map(|v| **v).sum();
    println!("{} {}", r.borrow().len(), total);

    let p = Rc::new(RefCell::new(vec![1]));
    let q = Rc::clone(&p);
    q.borrow_mut().push(2);
    drop(p);
    println!("{:?} {}", q.borrow(), Rc::strong_count(&q));
}
