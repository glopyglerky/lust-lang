// expect: L0311
// The reference from the coercion cannot outlive the guard.
use std::sync::Mutex;
struct L(u32);
fn main() {
    let m = Mutex::new(L(1));
    let g = m.lock().unwrap();
    let r: &L = &g;
    drop(g);
    println!("{}", r.0);
}
