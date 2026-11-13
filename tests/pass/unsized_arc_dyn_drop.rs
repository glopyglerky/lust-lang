use std::sync::Arc;
trait T { fn id(&self) -> u32; }
struct D(u32);
impl T for D { fn id(&self) -> u32 { self.0 } }
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn main() {
    let a: Arc<dyn T> = Arc::new(D(1));
    let b = a.clone();
    drop(a);
    println!("still {}", b.id());
    drop(b);
    let c: Arc<dyn T> = Arc::new(D(2));
    let _d = Arc::clone(&c);
    println!("end");
}
