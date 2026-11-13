// expect: L0612
use std::thread;
fn main() { let h = thread::spawn(|| std::rc::Rc::new(3)); println!("{}", h.join().unwrap()); }
