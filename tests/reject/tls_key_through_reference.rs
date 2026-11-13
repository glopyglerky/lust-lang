// expect: L0102
// Naming `std::thread::LocalKey` as a type (a key passed by reference) is
// not supported yet: key methods work on a `thread_local!` static by name.
use std::cell::Cell;
thread_local! {
    static S: Cell<i32> = Cell::new(4);
}
fn read(k: &'static std::thread::LocalKey<Cell<i32>>) -> i32 {
    k.get()
}
fn main() {
    println!("{}", read(&S));
}
