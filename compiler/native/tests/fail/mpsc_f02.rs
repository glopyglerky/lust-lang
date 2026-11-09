// expect: L0612
use std::sync::mpsc;
use std::thread;
fn main() { let (tx, rx) = mpsc::channel::<u8>(); let r = &rx; thread::scope(|s| { s.spawn(|| r.recv()); }); tx.send(1).unwrap(); }
