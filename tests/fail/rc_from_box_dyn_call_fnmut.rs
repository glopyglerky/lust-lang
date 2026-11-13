// expect: L0110
// rustc E0596: an `Rc<dyn FnMut()>` gives only a shared reference, so the
// closure cannot be called through it.
use std::rc::Rc;
fn main() {
    let mut n = 0;
    let b: Box<dyn FnMut() -> i32 + '_> = Box::new(|| {
        n += 1;
        n
    });
    let r: Rc<dyn FnMut() -> i32 + '_> = Rc::from(b);
    println!("{}", r());
}
