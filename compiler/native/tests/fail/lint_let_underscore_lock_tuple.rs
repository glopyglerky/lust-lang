// expect: L0544
// rustc let_underscore_lock: any `_` in a let pattern, and `_ = ..`.
use std::sync::RwLock;
fn main() {
    let m = RwLock::new(1);
    let (_, k) = (m.write().unwrap(), 2);
    println!("{}", k);
}
