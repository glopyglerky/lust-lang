// capability: types: Mutex, RwLock, Condvar, Barrier, atomics, mpsc channels, and std::thread (spawn, scope, Builder)
// expect: ok

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::thread;

fn main() {
    let m = Arc::new(Mutex::new(0u32));
    let hs: Vec<_> = (0..4)
        .map(|_| {
            let m = Arc::clone(&m);
            thread::spawn(move || *m.lock().unwrap() += 1)
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let rw = RwLock::new(5u32);
    *rw.write().unwrap() += 1;
    let n = AtomicU32::new(0);
    let bar = Barrier::new(2);
    thread::scope(|s| {
        s.spawn(|| {
            n.fetch_add(1, Ordering::SeqCst);
            bar.wait();
        });
        bar.wait();
    });
    let (tx, rx) = mpsc::channel();
    let b = thread::Builder::new().name("w".into()).spawn(move || tx.send(9u32).unwrap()).unwrap();
    b.join().unwrap();
    let pair = (Mutex::new(true), Condvar::new());
    let ready = pair.1.wait_while(pair.0.lock().unwrap(), |r| !*r).unwrap();
    println!("{} {} {} {} {}", m.lock().unwrap(), rw.read().unwrap(), n.load(Ordering::SeqCst), rx.recv().unwrap(), *ready);
}
