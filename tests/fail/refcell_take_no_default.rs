// expect: L0612
// take needs T: Default.
use std::cell::RefCell;
struct NoDefault(u8);
fn main() {
    let r = RefCell::new(NoDefault(1));
    let x = r.take();
    println!("{}", x.0);
}
