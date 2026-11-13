// expect: L0506
// `alloc` is not in the edition 2024 extern prelude: without
// `extern crate alloc;` at the crate root, the import is rustc E0433.
use alloc::vec::Vec;

fn main() {
    let v: Vec<i32> = Vec::new();
    println!("{}", v.len());
}
