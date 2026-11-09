// A `const { .. }` thread_local initializer with statements, ending in
// std's const `Cell::new`.
use std::cell::Cell;
thread_local! {
    static S: Cell<u32> = const {
        let x = 2;
        Cell::new(x * 3)
    };
}
fn main() {
    println!("{}", S.get());
}
