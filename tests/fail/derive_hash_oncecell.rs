// expect: L0110
// rustc E0277: `OnceCell<T>` does not implement `Hash`.
use std::cell::OnceCell;
#[derive(Hash)]
struct W {
    c: OnceCell<u32>,
}
fn main() {
    println!("x");
}
