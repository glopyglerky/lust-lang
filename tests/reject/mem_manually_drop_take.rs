// expect: L0404
// Valid Rust: ManuallyDrop::take needs unsafe, which lust does not support.
use std::mem::ManuallyDrop;
fn main() {
    let mut m = ManuallyDrop::new(String::from("x"));
    let s = unsafe { ManuallyDrop::take(&mut m) };
    println!("{s}");
}
