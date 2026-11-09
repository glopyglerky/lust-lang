// expect: L0112
use std::rc::Rc;
struct NoClone(u8);
fn main() {
    let a = Rc::new(NoClone(1));
    let b = Rc::unwrap_or_clone(a);
    println!("{}", b.0);
}
