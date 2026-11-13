// expect: L0612
// rustc E0277: core's reflexive `Borrow` holds at the first step, so Self is
// `&mut String`, which borrows as itself or `String`, never `str`.
use std::borrow::Borrow;
fn main() {
    let mut s = String::from("s");
    let m = &mut s;
    let o: &str = m.borrow();
    println!("{}", o);
}
