// expect: L0104
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel(); tx.send(1u8).unwrap(); let r2 = rx.clone(); }
