// expect: L0104
use std::sync::Mutex;
fn main() { let m = Mutex::new(1); let n = m.clone(); println!("{:?}", n); }
