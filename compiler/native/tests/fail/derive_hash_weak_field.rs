// expect: L0110
// `rc::Weak<T>` is Debug and Clone, but not Hash, for every T.
use std::cell::OnceCell;
use std::rc::Weak;
#[derive(Hash)]
struct S {
    w: Weak<i32>,
}
fn main() {
    let _ = OnceCell::from(S { w: Weak::new() });
}
