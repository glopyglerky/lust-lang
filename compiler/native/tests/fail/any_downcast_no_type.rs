// expect: L0610
// rustc E0282: nothing fixes the downcast target (W-CHECK-G3).
use std::any::Any;
fn main() { let b: Box<dyn Any> = Box::new(1); let r = b.downcast_ref(); println!("{}", r.is_some()); }
