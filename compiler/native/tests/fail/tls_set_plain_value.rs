// expect: L0104
// `set` exists only on `LocalKey<Cell<T>>` and `LocalKey<RefCell<T>>` (E0599).
thread_local! {
    static S: i32 = 1;
}
fn main() {
    S.set(5);
}
