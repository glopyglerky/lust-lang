// expect: L0306
// A key's value type is 'static: `set` cannot store a borrow of a local.
use std::cell::Cell;
thread_local! {
    static S: Cell<&'static str> = const { Cell::new("") };
}
fn main() {
    let s = String::from("x");
    S.set(&s);
    println!("{}", S.get());
}
