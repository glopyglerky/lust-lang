// expect: L0112
use std::sync::Mutex;
#[derive(Clone)]
struct S { m: Mutex<i32> }
fn main() { let s = S { m: Mutex::new(1) }; let _t = s.clone(); }
