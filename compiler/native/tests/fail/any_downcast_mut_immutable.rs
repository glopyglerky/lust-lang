// expect: L0308
// rustc E0596: `downcast_mut` borrows the box mutably (W-CHECK-G3).
use std::any::Any;
fn main() { let b: Box<dyn Any> = Box::new(1); if let Some(x) = b.downcast_mut::<i32>() { *x += 1; } }
