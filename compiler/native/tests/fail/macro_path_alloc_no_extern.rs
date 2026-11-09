// expect: L0802
// rustc E0433: `alloc` is in the extern prelude only after `extern crate alloc;`.
fn main() {
    let v: Vec<i32> = alloc::vec![1];
    println!("{v:?}");
}
