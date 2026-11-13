// expect: L0102
// A fn-body import is not visible in another fn.
fn main() {
    use std::rc::Rc;
    let b = Rc::new(2);
    println!("{}", b);
}
fn other() -> Rc<i32> {
    Rc::new(1)
}
