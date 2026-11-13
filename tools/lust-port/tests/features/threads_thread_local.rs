// feature: threads.thread_local
// form: thread_local! with a RefCell, as in ts-rust program.rs per-thread caches
use std::cell::RefCell;
thread_local! { static CACHE: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) }; }
fn main() {
    CACHE.with(|c| c.borrow_mut().push(3));
    CACHE.with(|c| println!("{}", c.borrow().len()));
}
