// expect: L0306
use std::sync::OnceLock;
fn main() { let o = OnceLock::new(); { let x = 5; o.get_or_init(|| &x); } println!("{:?}", o.get()); }
