// expect: L0306
// rustc E0597: a try_borrow guard borrows its RefCell.
use std::cell::RefCell;
fn main() {
    let g = {
        let c = RefCell::new(1);
        c.try_borrow().unwrap()
    };
    println!("{}", *g);
}
