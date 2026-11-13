// expect: L0306
// A RefMut guard writes into the cell's own T.
use std::cell::RefCell;
static S: i32 = 7;
fn run(c: &RefCell<&'static i32>) {
    let local = 5;
    let mut g = c.borrow_mut();
    *g = &local;
}
fn main() {
    let c = RefCell::new(&S);
    run(&c);
    println!("{}", c.borrow());
}
