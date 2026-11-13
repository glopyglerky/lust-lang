// `Type::clone(&x)` for the builtin containers. The Rc form
// must bump the count, not clone the pointee: an exact match
// beats the deref coercion.
use std::rc::Rc;
fn main() {
    let v = vec![1, 2];
    let w = Vec::clone(&v);
    println!("{:?}", w);
    let s = "x".to_string();
    let t = String::clone(&s);
    println!("{}", t);
    let a = Rc::new(3);
    let b = Rc::clone(&a);
    println!("{} {}", b, Rc::strong_count(&a));
    let bx = Box::new(4);
    let by = Box::clone(&bx);
    println!("{}", by);
}
