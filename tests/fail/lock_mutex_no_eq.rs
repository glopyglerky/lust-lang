// expect: L0112
use std::sync::Mutex;
fn main() { let m = Mutex::new(1); let b = Mutex::new(1); println!("{}", m == b); }
