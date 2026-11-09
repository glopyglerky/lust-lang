// expect: L0306
use std::sync::mpsc;
fn main() { let r; { let (tx, rx) = mpsc::channel::<u8>(); let s = String::from("a"); let (t2, r2) = mpsc::channel(); t2.send(&s).unwrap(); r = r2.recv().unwrap(); drop(tx); drop(rx); } println!("{}", r); }
