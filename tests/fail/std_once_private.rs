// expect: L0510
// rustc E0616: the cell's storage is private.
use std::cell::OnceCell;
fn main() {
    let o: OnceCell<i32> = OnceCell::new();
    println!("{:?}", o.inner);
}
