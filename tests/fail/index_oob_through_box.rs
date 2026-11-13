// expect: L0541
// rustc: unconditional_panic sees the array length through Box and Rc.
use std::rc::Rc;
fn main() {
    let b = Box::new([1u8, 2]);
    let r = Rc::new([1u8, 2]);
    println!("{}", r[1]);
    println!("{}", b[5]);
}
