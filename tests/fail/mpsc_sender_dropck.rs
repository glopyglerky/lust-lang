// expect: L0306
use std::sync::mpsc;
fn main() { let ch = mpsc::channel(); let tx = ch.0; let x = 5; tx.send(&x).unwrap(); }
