// `&S` in a fn-body static initializer binds to that fn's `S`.
use std::sync::atomic::{AtomicU32, Ordering};

fn first() -> u32 {
    static S: AtomicU32 = AtomicU32::new(1);
    static R: &AtomicU32 = &S;
    R.fetch_add(1, Ordering::SeqCst)
}

fn second() -> u32 {
    static S: AtomicU32 = AtomicU32::new(50);
    static R: &AtomicU32 = &S;
    R.fetch_add(1, Ordering::SeqCst)
}

fn main() {
    println!("{} {} {} {}", first(), second(), first(), second());
}
