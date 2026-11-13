// expect: L0301
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel::<u8>(); drop(tx); for x in rx { println!("{}", x); } let _ = rx.recv(); }
