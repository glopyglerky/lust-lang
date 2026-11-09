// expect: L0733
// rustc E0521: data_fn's `&Weak<T>` is higher-ranked, so it cannot be kept.
use std::rc::{Rc, Weak};
fn main() {
    let mut keep: Vec<&Weak<u8>> = Vec::new();
    let r = Rc::new_cyclic(|w| {
        keep.push(w);
        1u8
    });
    println!("{} {}", r, keep.len());
}
