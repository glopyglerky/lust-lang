// Rc::into_inner and Rc::unwrap_or_clone: the last strong Rc gives up
// its value, any other drops its count (with Weak refs alive).
use std::rc::Rc;
#[derive(Clone, Debug)]
struct S(String);
impl Drop for S {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn main() {
    let a = Rc::new(S(String::from("a")));
    let w = Rc::downgrade(&a);
    let b = Rc::clone(&a);
    println!("{:?}", Rc::into_inner(a).is_none());
    println!("{} {}", Rc::strong_count(&b), w.upgrade().is_some());
    let inner = Rc::into_inner(b);
    println!("{:?} {}", inner, w.upgrade().is_none());
    let c = Rc::new(S(String::from("c")));
    let d = Rc::clone(&c);
    let x = Rc::unwrap_or_clone(c);
    println!("{:?} {}", x, Rc::strong_count(&d));
    let y = Rc::unwrap_or_clone(d);
    println!("{:?}", y);
}
