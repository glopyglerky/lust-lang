// feature: panic.recovery
// form: a panicking worker thread's join Err and a poisoned Mutex, as in ts-rust's worker pools
// rustc-panic: unwind
use std::sync::{Arc, Mutex};
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let m = Arc::new(Mutex::new(1));
    let m2 = Arc::clone(&m);
    let r = std::thread::spawn(move || {
        let _g = m2.lock().unwrap();
        panic!("worker");
    })
    .join();
    println!("err={} poisoned={}", r.is_err(), m.is_poisoned());
}
