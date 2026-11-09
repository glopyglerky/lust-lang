// feature: std.rc
// form: plain Rc with clone and strong_count
use std::rc::Rc;
fn main() {
    let a: Rc<i32> = Rc::new(3);
    let b: Rc<i32> = Rc::clone(&a);
    println!("{} {}", *b, Rc::strong_count(&a));
}
