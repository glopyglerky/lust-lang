// expect: L0301
// rustc E0382: `Box::downcast` takes the box by value (W-CHECK-G3).
use std::any::Any;
fn main() { let b: Box<dyn Any> = Box::new(1); let _ = b.downcast::<i32>(); println!("{}", b.is::<i32>()); }
