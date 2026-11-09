// expect: L0113
use std::sync::Mutex;
fn main() { let m = Mutex::new(5); println!("{}", m.lock().unwrap() == 5); }
