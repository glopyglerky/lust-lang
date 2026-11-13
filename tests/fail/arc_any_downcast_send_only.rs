// expect: L0104
// std has `downcast` only on `Arc<dyn Any + Send + Sync>` (rustc E0599).
use std::any::Any;
use std::sync::Arc;

fn main() {
    let a: Arc<dyn Any + Send> = Arc::new(1);
    println!("{:?}", a.downcast::<i32>().is_ok());
}
