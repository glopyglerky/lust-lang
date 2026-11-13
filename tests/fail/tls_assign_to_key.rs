// expect: L0121
// A key is not a place to assign (E0070).
use std::cell::Cell;
thread_local! {
    static S: Cell<i32> = Cell::new(0);
}
fn main() {
    S = 5;
}
