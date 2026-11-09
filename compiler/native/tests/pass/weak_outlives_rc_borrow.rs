// A Weak of a borrowed value outlives the Rc it came from, as in Rust:
// downgrade shares T, not the borrow of its argument.
use std::rc::{Rc, Weak};
fn main() {
    let w: Weak<&i32>;
    let x = 5;
    {
        let r = Rc::new(&x);
        w = Rc::downgrade(&r);
        println!("{}", w.upgrade().is_some());
    }
    println!("{}", w.upgrade().is_none());
}
