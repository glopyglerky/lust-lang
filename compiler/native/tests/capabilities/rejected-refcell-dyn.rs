// capability: rejected: methods other than `borrow`/`borrow_mut` (`lock`) on a `RefCell<dyn Tr>` (`Mutex<dyn Tr>`)
// expect: L0641

use std::cell::RefCell;
use std::rc::Rc;

trait T {
    fn f(&self) -> u32;
}

struct A;

impl T for A {
    fn f(&self) -> u32 {
        1
    }
}

fn main() {
    let c: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(A));
    println!("{}", c.try_borrow().unwrap().f());
}
