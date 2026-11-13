// expect: L0104
// rustc E0133: ManuallyDrop::drop is an unsafe fn (lust has no unsafe).
use std::mem::ManuallyDrop;
fn main() {
    let mut m = ManuallyDrop::new(String::from("x"));
    ManuallyDrop::drop(&mut m);
}
