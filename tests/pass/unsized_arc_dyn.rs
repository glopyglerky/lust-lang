use std::sync::Arc;
trait T { fn f(&self) -> i32; }
struct A(i32);
impl T for A { fn f(&self) -> i32 { self.0 } }
fn main() { let a: Arc<dyn T> = Arc::new(A(5)); let b = a.clone(); println!("{} {}", b.f(), Arc::strong_count(&a)); }
