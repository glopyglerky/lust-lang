// expect: L0110
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel(); tx.send(1u8).unwrap(); let v: u32 = rx.recv().unwrap(); }
