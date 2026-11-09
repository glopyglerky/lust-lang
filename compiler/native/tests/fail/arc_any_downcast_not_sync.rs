// expect: L0612
// `Arc::downcast` bounds its target by `Send + Sync` (rustc E0277).
use std::any::Any;
use std::cell::Cell;
use std::sync::Arc;

fn main() {
    let a: Arc<dyn Any + Send + Sync> = Arc::new(1);
    println!("{:?}", a.downcast::<Cell<i32>>().is_ok());
}
