// expect: L0306
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel(); let x = 5; tx.send(&x).unwrap(); drop(x); let v = rx.recv().unwrap(); let _ = v; }
