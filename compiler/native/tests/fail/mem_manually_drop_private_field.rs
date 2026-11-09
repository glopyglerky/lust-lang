// expect: L0510
// rustc E0616: the field of ManuallyDrop is private.
use std::mem::ManuallyDrop;
fn main() {
    let m = ManuallyDrop::new(1);
    println!("{:?}", m.value);
}
