// expect: L0544
// rustc let_underscore_lock (deny): `let _ =` drops the guard at once.
use std::sync::Mutex;
fn main() {
    let m = Mutex::new(1);
    let _ = m.lock();
    println!("done");
}
