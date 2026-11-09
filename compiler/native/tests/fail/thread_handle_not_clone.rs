// expect: L0104
use std::thread;
fn main() { let h = thread::spawn(|| 1); let h2 = h.clone(); }
