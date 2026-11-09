// try_lock and try_write record the panic state like lock; a poisoned
// lock's try_lock and try_read are Err.
use std::sync::{Mutex, RwLock};
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let m = Mutex::new(1);
    println!("{}", m.try_lock().is_ok());
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _g = m.try_lock().unwrap(); panic!("p"); }));
    println!("{} {}", m.is_poisoned(), m.try_lock().is_ok());
    println!("{}", m.lock().is_err());
    let rw = RwLock::new(1);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _g = rw.try_write().unwrap(); panic!("w"); }));
    println!("{} {} {}", rw.is_poisoned(), rw.try_read().is_ok(), rw.try_write().is_ok());
}
