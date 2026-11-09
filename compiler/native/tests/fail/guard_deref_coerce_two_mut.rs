// expect: L0304
// The coercion borrows the guard: two `&mut` at once through one guard.
use std::sync::Mutex;
struct L(u32);
fn both(a: &mut L, b: &mut L) {
    a.0 += b.0;
}
fn main() {
    let m = Mutex::new(L(1));
    let mut g = m.lock().unwrap();
    let r: &mut L = &mut g;
    both(r, &mut g);
    println!("{}", g.0);
}
