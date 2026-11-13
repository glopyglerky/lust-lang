// expect: L0303
// rustc E0502: get_mut while a try_borrow_mut guard lives.
use std::cell::RefCell;
fn main() {
    let mut c = RefCell::new(1);
    let g = c.try_borrow_mut().unwrap();
    *c.get_mut() += 1;
    println!("{}", *g);
}
