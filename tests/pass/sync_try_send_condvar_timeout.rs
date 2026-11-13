// `SyncSender::try_send` with `TrySendError`, and
// `Condvar::wait_timeout` with `WaitTimeoutResult` (TRIAGE T5).
// Prints only facts that hold on every run.
use std::sync::mpsc::{TrySendError, sync_channel};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

fn read<T>(v: T) -> T {
    v
}

fn main() {
    let (tx, rx) = sync_channel::<i32>(1);
    println!("{:?}", tx.try_send(1));
    match tx.try_send(2) {
        Err(TrySendError::Full(v)) => println!("full {v}"),
        other => println!("{other:?}"),
    }
    println!("{:?}", rx.try_recv());
    println!("{:?}", tx.try_send(3));
    drop(rx);
    match tx.try_send(4) {
        Err(e) => println!("{:?} {} {:?}", e, e, e == TrySendError::Disconnected(4)),
        Ok(()) => println!("ok"),
    }
    let (tx0, rx0) = sync_channel::<i32>(0);
    println!("{:?}", tx0.try_send(5));
    drop(rx0);
    println!("{:?}", tx0.try_send(6));

    let m = Mutex::new(0);
    let c = Condvar::new();
    let g = m.lock().unwrap();
    let (g2, r) = c.wait_timeout(g, Duration::from_millis(20)).unwrap();
    println!("{} {} {:?}", *g2, r.timed_out(), r);
    let (_g3, r3) = c.wait_timeout_ms(g2, 20).unwrap();
    println!("{}", r3);
    if read(true) {
        c.notify_one();
    }
    println!("done");
}
