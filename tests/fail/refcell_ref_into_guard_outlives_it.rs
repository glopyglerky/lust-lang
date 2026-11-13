// expect: L0306
// A reference through a guard cannot outlive the guard.
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(vec![1]);
    let first: &i32;
    {
        let g = c.borrow();
        first = &g[0];
    }
    println!("{}", first);
}
