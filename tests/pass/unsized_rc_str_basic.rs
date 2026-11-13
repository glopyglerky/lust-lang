use std::rc::Rc;
fn main() {
    let a: Rc<str> = Rc::from("hello");
    let b = a.clone();
    println!("{} {} {}", a, b.len(), Rc::strong_count(&a));
    println!("{:?} {}", b, Rc::ptr_eq(&a, &b));
    drop(b);
    println!("{}", Rc::strong_count(&a));
}
