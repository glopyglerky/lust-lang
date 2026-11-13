// expect: L0306
// A push through a RefMut guard stores into the cell.
use std::cell::RefCell;
fn main() {
    let x = 5;
    let c = RefCell::new(vec![&x]);
    {
        let y = 6;
        c.borrow_mut().push(&y);
    }
    println!("{}", c.borrow().len());
}
