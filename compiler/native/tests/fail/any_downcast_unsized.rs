// expect: L0110
// rustc E0277: the downcast target must be sized (W-CHECK-G3).
use std::any::Any;
fn main() { let b: Box<dyn Any> = Box::new(1); println!("{}", b.downcast_ref::<str>().is_some()); }
