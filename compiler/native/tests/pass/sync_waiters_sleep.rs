// Threads that sleep in the runtime's futex waits: seven wait while an eighth sets a OnceLock
// slowly (the OnceLock waiter state), eight fight for one Mutex, a receiver sleeps until a
// send wakes it, a sender sleeps on a full sync_channel, and a recv_timeout sleeps until a send.
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static SLOW: OnceLock<u64> = OnceLock::new();

fn main() {
    let hs: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                *SLOW.get_or_init(|| {
                    thread::sleep(Duration::from_millis(30));
                    42
                })
            })
        })
        .collect();
    let v: Vec<u64> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    println!("once {:?}", v);

    let m = Arc::new(Mutex::new(0u64));
    let hs: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&m);
            thread::spawn(move || {
                for _ in 0..20000 {
                    *m.lock().unwrap() += 1;
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    println!("mutex {}", *m.lock().unwrap());

    let (tx, rx) = mpsc::channel::<u32>();
    let r = thread::spawn(move || (0..3).map(|_| rx.recv().unwrap()).sum::<u32>());
    for i in 1..=3 {
        thread::sleep(Duration::from_millis(10));
        tx.send(i).unwrap();
    }
    println!("recv {}", r.join().unwrap());

    let (tx, rx) = mpsc::sync_channel::<u32>(1);
    let s = thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });
    thread::sleep(Duration::from_millis(20));
    let got: Vec<u32> = rx.iter().collect();
    s.join().unwrap();
    println!("sync {:?}", got);

    let (tx, rx) = mpsc::channel::<u32>();
    let t = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        tx.send(7).unwrap();
    });
    println!("timeout {:?}", rx.recv_timeout(Duration::from_secs(5)));
    t.join().unwrap();
}
