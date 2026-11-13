// TryLockError is std's enum: Poisoned(PoisonError<T>) and WouldBlock, by
// path, by glob import, and in patterns, with std's Debug and Display.
use std::sync::{Mutex, PoisonError, RwLock, TryLockError};

fn state<G>(r: &Result<G, TryLockError<G>>) -> &'static str {
    match r {
        Ok(_) => "ok",
        Err(TryLockError::WouldBlock) => "would block",
        Err(TryLockError::Poisoned(_)) => "poisoned",
    }
}

fn main() {
    let m = Mutex::new(vec![1, 2]);
    let held = m.try_lock();
    let again = m.try_lock();
    println!("{} {}", state(&held), state(&again));
    if let Err(e) = &again {
        println!("{e:?} | {e} | [{e:>55}] | [{e:<8.6}]");
    }
    println!("{}", matches!(again, Err(TryLockError::WouldBlock)));
    drop(held);
    println!("{:?}", m.try_lock().map(|g| g.len()));

    let rw = RwLock::new(3);
    let r1 = rw.try_read();
    let r2 = rw.try_read();
    let w = rw.try_write();
    println!("{} {} {}", state(&r1), state(&r2), state(&w));
    drop((r1, r2));
    println!("{}", state(&rw.try_write()));

    let p: TryLockError<String> = TryLockError::Poisoned(PoisonError::new(String::from("kept")));
    println!("{p:?} | {p}");
    match p {
        TryLockError::Poisoned(e) => println!("{}", e.into_inner()),
        TryLockError::WouldBlock => println!("unreachable"),
    }
    {
        use std::sync::TryLockError::*;
        let e: TryLockError<u8> = WouldBlock;
        println!("{}", if let WouldBlock = e { "glob" } else { "no" });
    }
    println!("{}", std::any::type_name::<TryLockError<std::sync::MutexGuard<'_, u8>>>());
}
