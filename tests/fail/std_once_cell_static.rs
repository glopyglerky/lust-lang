// expect: L0612
// rustc E0277: `OnceCell` is not `Sync`, so it cannot be a shared static.
use std::cell::OnceCell;
static C: OnceCell<i32> = OnceCell::new();
fn main() {
    println!("{:?}", C.get());
}
