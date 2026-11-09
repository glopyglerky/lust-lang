// expect: L0309
use std::rc::Rc;
fn main() {
    let mut a = Rc::new(1);
    *a = 2;
    println!("{}", a);
}
