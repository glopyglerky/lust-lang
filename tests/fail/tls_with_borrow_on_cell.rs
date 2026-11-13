// expect: L0104
// `with_borrow` is a `LocalKey<RefCell<T>>` method only (E0599).
use std::cell::Cell;
thread_local! {
    static S: Cell<u8> = Cell::new(1);
}
fn main() {
    S.with_borrow(|v| println!("{}", v));
}
