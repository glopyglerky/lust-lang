// expect: L0112
// rustc E0277: `OnceCell<T>: Clone` needs `T: Clone`, so the
// derived Clone fails too.
use std::cell::OnceCell;
struct NoClone(u8);
#[derive(Clone)]
struct W {
    c: OnceCell<NoClone>,
}
fn main() {
    let w = W { c: OnceCell::new() };
    let _ = w.clone();
    println!("x");
}
