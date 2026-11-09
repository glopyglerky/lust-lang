// expect: L0612
use std::sync::mpsc;
use std::rc::Rc;
use std::thread;
fn main() { let (tx, rx) = mpsc::channel(); thread::spawn(move || tx.send(Rc::new(1)).unwrap()); println!("{}", rx.recv().unwrap()); }
