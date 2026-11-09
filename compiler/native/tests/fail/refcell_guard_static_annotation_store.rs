// expect: L0306
// A RefMut guard of a 'static RefCell cannot store a local borrow.
use std::cell::RefCell;
static S: i32 = 7;
fn main() {
    let c: RefCell<&'static i32> = RefCell::new(&S);
    {
        let local = 5;
        let mut g = c.borrow_mut();
        *g = &local;
    }
    println!("{}", c.borrow());
}
