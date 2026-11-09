// expect: L0309
use std::sync::RwLock;
fn main() { let r = RwLock::new(1); *r.read().unwrap() = 5; println!("{:?}", r); }
