// Poisoning: only a write guard dropped by a panic poisons (Mutex,
// RwLock write); a read guard and a guard dropped normally do not;
// into_inner and lock report it; Debug shows poisoned.
use std::sync::{Arc, Mutex, RwLock};
use std::panic;
fn main() {
    panic::set_hook(Box::new(|_| {}));
    let m = Arc::new(Mutex::new(vec![1]));
    let r = panic::catch_unwind(panic::AssertUnwindSafe(|| { let mut g = m.lock().unwrap(); g.push(2); panic!("x"); }));
    println!("{} {}", r.is_err(), m.is_poisoned());
    println!("{:?}", m.lock());
    let m2 = Mutex::new(1);
    let r = panic::catch_unwind(|| { let g = m2.lock().unwrap(); drop(g); });
    println!("{} {}", r.is_ok(), m2.is_poisoned());
    let rw = RwLock::new(3);
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { let _r = rw.read().unwrap(); panic!("r"); }));
    println!("read poison {}", rw.is_poisoned());
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { let _w = rw.write().unwrap(); panic!("w"); }));
    println!("write poison {}", rw.is_poisoned());
    println!("{:?}", rw.read().is_err());
    println!("{:?}", Mutex::new(1).into_inner());
    let m3 = Mutex::new(9);
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { let _g = m3.lock().unwrap(); panic!("p"); }));
    println!("{:?}", m3.into_inner().is_err());
    println!("{:?}", m);
}
