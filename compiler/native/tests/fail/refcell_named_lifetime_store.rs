// expect: L0306
// RefCell is invariant through a RefMut guard.
use std::cell::RefCell;
fn put<'a>(c: &RefCell<&'a i32>, x: &'a i32) {
    *c.borrow_mut() = x;
}
static S: i32 = 7;
fn main() {
    let c: RefCell<&'static i32> = RefCell::new(&S);
    {
        let local = 5;
        put(&c, &local);
    }
    println!("{}", c.borrow());
}
