// expect: L0110
// `impl<T> Default for sync::Weak<T>` needs `T: Sized`: a
// `Weak<dyn Fn()>` field has no default (R26 sweep false accept).
use std::sync::{Arc, Weak};
#[derive(Default)]
struct Inner {
    w: (Weak<dyn Fn()>, i32),
}
#[derive(Default)]
struct Outer {
    h: Option<Inner>,
}
fn main() {
    let r: Arc<dyn Fn()> = Arc::new(|| ());
    let o = Outer { h: Some(Inner { w: (Arc::downgrade(&r), 1) }) };
    let _ = Outer::default();
    println!("{}", o.h.is_some());
}
