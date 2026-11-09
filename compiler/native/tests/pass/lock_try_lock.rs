use std::sync::{Mutex, RwLock, TryLockError};
fn main() {
    let m = Mutex::new(5);
    {
        let g = m.lock().unwrap();
        let t = m.try_lock();
        println!("{} {:?} {}", t.is_err(), t.as_ref().err(), *g);
        if let Err(e) = &t { println!("{e} {e:?}"); }
    }
    match m.try_lock() { Ok(mut g) => { *g += 1; println!("got {}", *g) } Err(_) => println!("busy") }
    let r = RwLock::new(vec![1]);
    let a = r.try_read().unwrap();
    let b = r.try_read().unwrap();
    println!("{} {} {}", a.len(), b.len(), r.try_write().is_err());
    drop(a); drop(b);
    r.try_write().unwrap().push(2);
    println!("{:?}", r);
}
