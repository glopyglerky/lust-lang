use std::sync::Arc;
trait W { fn w(&self) -> u8; }
struct A;
impl W for A { fn w(&self) -> u8 { 1 } }
fn take(x: Arc<dyn W>) -> u8 { x.w() }
fn main() {
    let s: Arc<dyn W + Send + Sync> = Arc::new(A);
    let t: Arc<dyn W> = s.clone();
    let a = Arc::new(A);
    println!("{} {} {}", take(t), take(a.clone() as Arc<dyn W>), take(a));
}
