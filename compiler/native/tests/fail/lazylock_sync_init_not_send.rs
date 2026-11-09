// expect: L0612
// std: `LazyLock<T, F>: Sync` needs `F: Send`; a closure holding a
// `MutexGuard` is `Sync` but not `Send`.
use std::sync::{LazyLock, Mutex};
fn need_sync<T: Sync>(_: &T) {}
fn main() {
    let m = Mutex::new(1u8);
    let g = m.lock().unwrap();
    let l = LazyLock::new(move || *g + 1);
    need_sync(&l);
    println!("{}", LazyLock::force(&l));
}
