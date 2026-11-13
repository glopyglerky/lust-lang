// expect: L0301
use std::sync::mpsc;
fn main() { let (tx, rx) = mpsc::channel(); let s = String::from("x"); tx.send(s).unwrap(); println!("{} {:?}", s, rx.recv()); }
