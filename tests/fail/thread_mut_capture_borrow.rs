// expect: L0306
use std::thread;
fn main() { let mut n = 0; let h = thread::spawn(|| { n += 1; }); h.join().unwrap(); println!("{}", n); }
