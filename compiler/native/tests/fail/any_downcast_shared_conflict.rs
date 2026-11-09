// expect: L0303
// rustc E0502: `downcast_ref` and then `downcast_mut` on one box while the first is live (W-CHECK-G3).
use std::any::Any;
fn main() { let mut b: Box<dyn Any> = Box::new(1); let x = b.downcast_ref::<i32>(); let y = b.downcast_mut::<i32>(); println!("{} {}", x.is_some(), y.is_some()); }
