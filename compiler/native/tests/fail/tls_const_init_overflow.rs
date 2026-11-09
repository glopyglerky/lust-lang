// expect: L0630
// The const form evaluates at compile time in rustc: an overflow inside a
// const fn it calls fails the build (E0080).
use std::cell::Cell;
const fn double(x: u32) -> u32 {
    x * 2
}
thread_local! {
    static S: Cell<u32> = const { Cell::new(double(u32::MAX)) };
}
fn main() {
    println!("{}", S.get());
}
