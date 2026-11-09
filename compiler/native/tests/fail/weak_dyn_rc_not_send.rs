// expect: L0612
// `rc::Weak<dyn Tr + Send + Sync>` is never `Send`.
use std::rc::{Rc, Weak};
trait Tr: Send + Sync {
    fn v(&self) -> u32;
}
struct A;
impl Tr for A {
    fn v(&self) -> u32 {
        1
    }
}
fn mk() -> Rc<dyn Tr> {
    Rc::new(A)
}
fn main() {
    let a = mk();
    let w: Weak<dyn Tr> = Rc::downgrade(&a);
    let h = std::thread::spawn(move || w.upgrade().is_some());
    println!("{}", h.join().unwrap());
}
