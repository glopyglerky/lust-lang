// Debug for dyn Any prints `Any { .. }` for every flag (W-CHECK-G3).
use std::any::Any;
fn main() { let b: Box<dyn Any> = Box::new(1); println!("[{:?}] [{:#?}] [{:>15?}] [{:?}]", b, b, b, Some(&*b)); let v: Vec<Box<dyn Any + Send + Sync>> = vec![Box::new(2)]; println!("{:#?}", v); }
