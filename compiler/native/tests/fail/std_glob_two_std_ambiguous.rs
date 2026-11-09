// expect: L0507
// rustc E0659: `std::rc::*` and `std::sync::*` both bring a `Weak`.
use std::rc::*;
use std::sync::*;
fn main() {
    let w: Weak<i32> = Weak::new();
    println!("{}", w.upgrade().is_none());
}
