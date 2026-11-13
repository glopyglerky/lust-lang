// expect: L0306
// Guards in a Vec keep their RefCell borrowed until the Vec drops.
use std::cell::RefCell;
fn main() {
    let mut v = Vec::new();
    {
        let c = RefCell::new(1);
        v.push(c.borrow());
    }
    println!("{}", v.len());
}
