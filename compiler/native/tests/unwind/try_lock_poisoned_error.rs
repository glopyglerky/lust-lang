// A poisoned lock's try_lock/try_read/try_write is Err(TryLockError::Poisoned(guard)):
// the guard keeps the lock (a second try is WouldBlock), into_inner gives it back,
// and Debug/Display are std's texts.
use std::sync::{Mutex, RwLock, TryLockError};
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let m = Mutex::new(vec![1]);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _g = m.lock().unwrap(); panic!("p"); }));
    let r = m.try_lock();
    match &r { Ok(_) => println!("ok"), Err(e) => println!("{:?} / {}", e, e) }
    let r2 = m.try_lock();
    match &r2 { Ok(_) => println!("ok"), Err(e) => println!("{:?} / {} / {:>60}|", e, e, e) }
    drop(r2);
    drop(r);
    if let Err(TryLockError::Poisoned(p)) = m.try_lock() { println!("{:?}", p.get_ref().len()); let mut g = p.into_inner(); g.push(2); println!("{:?}", *g); }
    match m.try_lock() { Err(TryLockError::WouldBlock) => println!("wb"), Err(TryLockError::Poisoned(e)) => println!("p {:?}", e), Ok(_) => println!("ok") }
    m.clear_poison();
    println!("{}", m.is_poisoned());
    println!("{:?}", m.try_lock().map(|g| g.len()));
    let rw = RwLock::new(3);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _g = rw.write().unwrap(); panic!("w"); }));
    let a = rw.try_read();
    println!("{:?}", a.as_ref().err().map(|e| format!("{e}")));
    let b = rw.try_read();
    println!("{:?} {:?}", b.is_ok(), b.err().map(|e| format!("{e:?}")));
    let c = rw.try_write();
    println!("{:?}", c.err().map(|e| format!("{e:?}")));
    drop(a);
    let e: TryLockError<i32> = TryLockError::WouldBlock;
    println!("{e:?} {e}");
    println!("{}", std::any::type_name::<TryLockError<std::sync::MutexGuard<'_, u8>>>());
}
