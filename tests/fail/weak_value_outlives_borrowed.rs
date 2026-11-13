// expect: L0306
use std::rc::{Rc, Weak};
fn main() {
    let w: Weak<&i32>;
    {
        let y = 6;
        let r = Rc::new(&y);
        w = Rc::downgrade(&r);
        println!("{}", w.upgrade().is_some());
    }
    println!("{}", w.upgrade().is_none());
}
