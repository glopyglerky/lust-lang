// expect: L0110
use std::sync::Arc;
use std::rc::Weak;
fn main() { let a = Arc::new(3); let w: Weak<i32> = Arc::downgrade(&a); println!("{:?}", w.upgrade()); }
