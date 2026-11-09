// `LazyLock::new(INIT)` and `LazyCell::new(INIT)` where INIT is a path with
// a `crate`, `super` or `self` segment, also to a generic fn whose type
// argument comes from the static's type (ts-rust api/session_p1.rs
// SOURCE_FILE_SYMBOL_INDEX_KEY).
use std::sync::atomic::{AtomicU64, Ordering};
mod k {
    use super::*;
    pub static COUNTER: AtomicU64 = AtomicU64::new(0);
    pub struct Key<T> {
        pub key: u64,
        _t: std::marker::PhantomData<fn() -> T>,
    }
    pub fn new_key<T>() -> Key<T> {
        Key { key: COUNTER.fetch_add(1, Ordering::SeqCst) + 1, _t: std::marker::PhantomData }
    }
    pub fn seven() -> u32 {
        7
    }
}
static A: std::sync::LazyLock<
    k::Key<std::rc::Rc<Vec<u32>>>,
> = std::sync::LazyLock::new(crate::k::new_key);
static B: std::sync::LazyLock<k::Key<String>> = std::sync::LazyLock::new(self::k::new_key);
mod q {
    pub static C: std::sync::LazyLock<u32> = std::sync::LazyLock::new(super::k::seven);
}
thread_local! {
    static D: std::cell::LazyCell<u32> = std::cell::LazyCell::new(crate::k::seven);
}
fn main() {
    println!("{} {} {}", B.key, A.key, B.key);
    println!("{} {}", *q::C, D.with(|d| **d));
}
