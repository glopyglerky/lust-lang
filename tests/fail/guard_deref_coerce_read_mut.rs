// expect: L0110
// A read guard has no `DerefMut`: `&mut RwLockReadGuard<L>` is not `&mut L`.
use std::sync::RwLock;
struct L(u32);
fn take(l: &mut L) {
    l.0 += 1;
}
fn main() {
    let rw = RwLock::new(L(1));
    let mut r = rw.read().unwrap();
    take(&mut r);
    println!("{}", r.0);
}
