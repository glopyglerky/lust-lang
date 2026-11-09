// feature: std.rc_refcell
// form: Weak downgrade and upgrade on a plain Rc
use std::rc::{Rc, Weak};
fn main() {
    let strong: Rc<i32> = Rc::new(5);
    let weak: Weak<i32> = Rc::downgrade(&strong);
    let up: i32 = match weak.upgrade() { Some(v) => *v, None => 0 };
    println!("{}", up);
}
