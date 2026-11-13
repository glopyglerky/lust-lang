// capability: types: Box<T>, Rc<T>, Arc<T>, and their Weak pointers, also of `str`, `[T]`, and `dyn Trait`
// capability: types: Cell<T> and RefCell<T>
// expect: ok

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    let b: Box<u32> = Box::new(1);
    let bs: Box<str> = "ab".into();
    let rc: Rc<[u32]> = Rc::from(vec![1, 2, 3]);
    let w = Rc::downgrade(&rc);
    let a: Arc<str> = Arc::from("xyz");
    let aw = Arc::downgrade(&a);
    let cell = Cell::new(1u32);
    cell.set(cell.get() + 1);
    let shared = Rc::new(RefCell::new(vec![1u32]));
    shared.borrow_mut().push(2);
    println!("{} {} {} {} {} {} {:?}", b, bs, rc.len(), w.upgrade().is_some(), aw.upgrade().unwrap(), cell.get(), shared.borrow());
}
