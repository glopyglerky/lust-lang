// expect: L0110
// rustc E0277: alloc's `impl<T> Default for Weak<T>` needs `T: Sized`.
use std::rc::Weak;
#[derive(Default)]
struct S {
    w: Weak<dyn Fn()>,
}
fn main() {
    let s = S::default();
    println!("{}", s.w.upgrade().is_none());
}
