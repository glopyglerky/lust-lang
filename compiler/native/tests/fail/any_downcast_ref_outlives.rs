// expect: L0306
// rustc E0597: the downcast reference borrows the box (W-CHECK-G3).
use std::any::Any;
fn main() { let r; { let b: Box<dyn Any> = Box::new(1); r = b.downcast_ref::<i32>().unwrap(); } println!("{}", r); }
