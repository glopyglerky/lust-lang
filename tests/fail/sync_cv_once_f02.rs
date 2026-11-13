// expect: L0306
use std::sync::OnceLock;
fn main() { let o = OnceLock::new(); { let x = 5; let _ = o.set(&x); } println!("{:?}", o.get()); }
