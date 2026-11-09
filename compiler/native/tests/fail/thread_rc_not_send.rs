// expect: L0612
use std::thread;
use std::rc::Rc;
fn main() { let v = Rc::new(1); let h = thread::spawn(move || { let w = v.clone(); 2 }); println!("{}", h.join().unwrap()); }
