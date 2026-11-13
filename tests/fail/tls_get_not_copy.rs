// expect: L0612
// `LocalKey<Cell<T>>::get` needs `T: Copy` (E0277).
use std::cell::Cell;
thread_local! {
    static S: Cell<String> = Cell::new(String::new());
}
fn main() {
    let _x = S.get();
}
