// expect: L0802
// rustc E0433: the `extern crate alloc;` is cfg-d out.
#[cfg(any())]
extern crate alloc;
fn main() {
    let v: Vec<i32> = alloc::vec![1];
    println!("{v:?}");
}
