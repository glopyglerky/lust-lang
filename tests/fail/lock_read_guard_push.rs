// expect: L0308
use std::sync::RwLock;
fn main() { let r = RwLock::new(vec![1]); let g = r.read().unwrap(); g.push(2); }
