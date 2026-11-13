// RefCell::try_borrow and try_borrow_mut (W-STD-B): the guard in Ok, BorrowError and BorrowMutError (Debug and Display) where borrow would panic.
use std::cell::{BorrowError, BorrowMutError, RefCell};
fn main() {
    let c = RefCell::new(5);
    {
        let r = c.borrow();
        let e: BorrowMutError = c.try_borrow_mut().unwrap_err();
        println!("{:?} | {} | {}", e, e, *r);
        println!("{}", c.try_borrow().is_ok());
    }
    {
        let _w = c.borrow_mut();
        let e: BorrowError = c.try_borrow().unwrap_err();
        println!("{:?} | {}", e, e);
    }
    match c.try_borrow_mut() {
        Ok(mut g) => *g += 1,
        Err(_) => println!("busy"),
    }
    if let Ok(g) = c.try_borrow() {
        println!("{}", *g);
    }
}
