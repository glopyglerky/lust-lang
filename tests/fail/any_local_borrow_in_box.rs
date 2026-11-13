// expect: L0306
// rustc E0597: a borrow of a local in `Box<dyn Any>` must be 'static (W-CHECK-G3).
use std::any::Any;
fn main() { let mut v = 5; let b: Box<dyn Any> = Box::new(&mut v); println!("{}", b.is::<&mut i32>()); }
