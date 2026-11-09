// expect: L0306
use std::sync::Mutex;
fn main() { let m: Mutex<&'static str> = Mutex::new("a"); let s = String::from("b"); *m.lock().unwrap() = &s; drop(s); println!("{}", m.lock().unwrap()); }
