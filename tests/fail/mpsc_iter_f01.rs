// expect: L0306
use std::sync::mpsc;
fn main() { let it; { let (tx, rx) = mpsc::channel::<u8>(); drop(tx); it = rx.iter(); } for x in it { println!("{}", x); } }
