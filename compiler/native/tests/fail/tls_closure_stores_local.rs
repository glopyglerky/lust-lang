// expect: L0306
// rustc E0597: the value a `with` closure sees is 'static inside (`Cell<&'static str>`):
// a local's borrow cannot go into it.
use std::cell::Cell;
thread_local! {
    static S: Cell<&'static str> = const { Cell::new("") };
}
fn main() {
    S.with(|c| {
        let s = String::from("x");
        c.set(&s);
    });
}
