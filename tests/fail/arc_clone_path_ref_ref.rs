// expect: L0110
// rustc E0308: `Arc::clone(&&a)` returns an `Arc<i32>`.
use std::sync::Arc;
fn main() {
    let a = Arc::new(1);
    let w: &Arc<i32> = Arc::clone(&&a);
    println!("{:?}", w);
}
