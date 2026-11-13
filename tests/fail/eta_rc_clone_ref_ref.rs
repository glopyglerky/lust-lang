// expect: L0110
// rustc E0631: `Rc::clone` takes `&Rc<T>`; the items are `&&Rc<i32>`.
use std::rc::Rc;
fn main() {
    let a = Rc::new(1);
    let v = vec![&a];
    let w: Vec<&Rc<i32>> = v.iter().map(Rc::clone).collect();
    println!("{:?}", w);
}
