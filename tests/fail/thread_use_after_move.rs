// expect: L0301
use std::thread;
fn main() { let s = String::from("x"); let h = thread::spawn(move || s.len()); println!("{} {}", h.join().unwrap(), s); }
