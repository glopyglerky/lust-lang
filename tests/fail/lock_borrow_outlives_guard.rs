// expect: L0306
use std::sync::Mutex;
fn main() { let m = Mutex::new(vec![1]); let r: &Vec<i32>; { let g = m.lock().unwrap(); r = &*g; } println!("{:?}", r); }
