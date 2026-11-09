// expect: L0110
// `&MutexGuard<L>` does not coerce to `&mut L`.
use std::sync::{Mutex, MutexGuard};
struct L(u32);
fn take(l: &mut L) {
    l.0 += 1;
}
fn via(g: &MutexGuard<'_, L>) {
    take(g);
}
fn main() {
    let m = Mutex::new(L(1));
    let g = m.lock().unwrap();
    via(&g);
    println!("{}", g.0);
}
