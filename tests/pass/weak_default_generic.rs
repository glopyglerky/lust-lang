// `T::default()` and `Default::default()` at `T = sync::Weak<_>` passed
// check and failed at build (L0641): Weak's Default is `Weak::new()`.
use std::sync::{Arc, Weak};
fn d<T: Default>() -> T {
    T::default()
}
fn main() {
    let w: Weak<i32> = d();
    println!("{:?} {}", w.upgrade(), w.strong_count());
    let v: Weak<String> = Default::default();
    println!("{:?} {}", v.upgrade(), v.weak_count());
    let a = Arc::new(5u8);
    let mut x: Weak<u8> = d();
    println!("{:?}", x.upgrade());
    x = Arc::downgrade(&a);
    println!("{:?} {}", x.upgrade(), Arc::weak_count(&a));
}
