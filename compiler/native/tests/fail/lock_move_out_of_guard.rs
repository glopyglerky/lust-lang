// expect: L0305
use std::sync::Mutex;
fn main() { let m = Mutex::new(String::from("a")); let s = *m.lock().unwrap(); println!("{s}"); }
