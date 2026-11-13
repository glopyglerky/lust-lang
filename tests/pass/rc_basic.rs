// Rc::new, clone as a count bump, strong_count, drop by scope.
use std::rc::Rc;
fn main() {
    let a = Rc::new(7);
    let b = a.clone();
    println!("{} {} {}", *a, *b, Rc::strong_count(&a));
    drop(b);
    println!("{}", Rc::strong_count(&a));
    let s = Rc::new("shared".to_string());
    println!("{} {:?}", s.len(), s);
}
