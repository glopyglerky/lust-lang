// feature: std.rc_refcell
// form: Cell and OnceCell as plain locals
use std::cell::{Cell, OnceCell};
fn main() {
    let c: Cell<i32> = Cell::new(1);
    c.set(c.get() + 4);
    let once: OnceCell<i32> = OnceCell::new();
    let _ = once.set(9);
    println!("{} {}", c.get(), match once.get() { Some(v) => *v, None => 0 });
}
