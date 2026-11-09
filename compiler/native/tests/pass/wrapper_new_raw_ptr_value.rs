// One-value wrapper constructors coerce a reference to the expected raw pointer value
// (`Cell::new(&x)` for `Cell<*const u8>`); the generated C had a unit-typed temporary before.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Mutex;

fn main() {
    let x = 5u8;
    let mut y = 6u8;
    let p: *const u8 = &x;
    let c: Cell<*const u8> = Cell::new(&x);
    println!("{} {}", c.get().is_null(), c.get() == p);
    let b: Box<*const u8> = Box::new(&x);
    let r: Rc<*const u8> = Rc::new(&x);
    let m: Mutex<*mut u8> = Mutex::new(&mut y);
    let rc: RefCell<*const u8> = RefCell::new(&mut y);
    println!("{} {} {} {}", *b == p, *r == p, m.lock().unwrap().is_null(), rc.borrow().is_null());
}
