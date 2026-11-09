// rc::Weak::as_ptr (ts-rust ls/import_tracker.rs:222).
use std::cell::RefCell;
use std::rc::{Rc, Weak};
struct E { checker: Weak<RefCell<i32>>, n: u8 }
fn main() {
    let r = Rc::new(RefCell::new(5));
    let w: Weak<RefCell<i32>> = Rc::downgrade(&r);
    let key: *const RefCell<i32> = Rc::as_ptr(&r);
    let es = vec![E { checker: Weak::new(), n: 1 }, E { checker: w.clone(), n: 2 }];
    let hit = es.iter().find(|e| std::ptr::eq(e.checker.as_ptr(), key)).map(|e| e.n);
    println!("{:?} {}", hit, std::ptr::eq(w.as_ptr(), Rc::as_ptr(&r)));
    let d: Weak<u64> = Weak::new();
    println!("{}", d.as_ptr() as usize == usize::MAX);
    drop(r);
    println!("{}", std::ptr::eq(w.as_ptr(), key));
}
