use std::sync::Arc;
trait Base { fn base(&self) -> u32; }
trait Derived: Base { fn derived(&self) -> u32 { self.base() * 10 } }
struct S;
impl Base for S { fn base(&self) -> u32 { 4 } }
impl Derived for S {}
fn main() {
    let d: Arc<dyn Derived> = Arc::new(S);
    println!("{} {}", d.base(), d.derived());
    let b: Arc<dyn Base> = d.clone();
    println!("{} {}", b.base(), Arc::strong_count(&d));
}
