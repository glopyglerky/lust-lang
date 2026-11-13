// expect: L0306
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel(); let s = String::from("a"); tx.send(&s).unwrap(); drop(tx); let mut it = rx.into_iter(); drop(s); let _ = it.next(); }
