// expect: L0733
// A borrowed parameter does not live for 'static either.
use std::cell::Cell;
thread_local! {
    static S: Cell<&'static str> = const { Cell::new("") };
}
fn keep(x: &str) {
    S.set(x);
}
fn main() {
    keep("a");
}
