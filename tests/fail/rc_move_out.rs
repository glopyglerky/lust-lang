// expect: L0305
use std::rc::Rc;
fn main() {
    let a = Rc::new("x".to_string());
    let s = *a;
    println!("{}", s);
}
