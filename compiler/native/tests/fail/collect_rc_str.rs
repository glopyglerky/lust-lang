// expect: L0612
// rustc E0277: std has no `FromIterator` impl for `Rc<str>` (collect a String, then convert it).
use std::rc::Rc;
fn main() {
    let r: Rc<str> = "abc".chars().collect();
    println!("{r}");
}
