// expect: L0110
use std::sync::Arc;
use std::rc::Rc;
fn main() { let a: Arc<i32> = Rc::new(3); println!("{}", a); }
