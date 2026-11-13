// expect: L0631
// A `const { .. }` block with statements is still a const context: a call
// of a plain fn rejects (E0015).
use std::cell::Cell;
fn g() -> u32 {
    2
}
thread_local! {
    static S: Cell<u32> = const {
        let x = g();
        Cell::new(x)
    };
}
fn main() {
    println!("{}", S.get());
}
