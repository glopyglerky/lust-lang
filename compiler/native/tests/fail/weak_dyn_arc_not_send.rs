// expect: L0612
// `sync::Weak<dyn Tr>` is `Send` only when `dyn Tr` is `Send + Sync`.
use std::sync::{Arc, Weak};
trait Tr {
    fn v(&self) -> u32;
}
struct A;
impl Tr for A {
    fn v(&self) -> u32 {
        1
    }
}
fn mk() -> Arc<dyn Tr> {
    Arc::new(A)
}
fn main() {
    let a = mk();
    let w: Weak<dyn Tr> = Arc::downgrade(&a);
    let h = std::thread::spawn(move || w.upgrade().is_some());
    println!("{}", h.join().unwrap());
}
