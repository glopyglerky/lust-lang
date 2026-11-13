// Cell, RefCell, and rc::Weak basics (W-STD-A phase 3): values, borrows,
// counts (Rust keeps one implicit weak ref while strong refs live), Debug
// output, and the value drop while weak refs remain.
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}
fn main() {
    let c = Cell::new(5);
    c.set(c.get() + 1);
    let old = c.replace(10);
    println!("{} {} {:?}", old, c.get(), c);
    let r = RefCell::new(vec![1, 2]);
    r.borrow_mut().push(3);
    {
        let a = r.borrow();
        let b = r.borrow();
        println!("{} {} {:?}", a.len(), b[0], r);
    }
    let n = r.borrow().len();
    println!("{} {:?}", n, r.replace(vec![9]));
    println!("{:?}", r.into_inner());
    let shared = Rc::new(RefCell::new(String::from("hi")));
    let s2 = Rc::clone(&shared);
    s2.borrow_mut().push_str(" there");
    println!("{} {}", shared.borrow(), Rc::strong_count(&shared));
    let w: Weak<RefCell<String>> = Rc::downgrade(&shared);
    println!("{} {} {}", Rc::weak_count(&shared), w.strong_count(), w.weak_count());
    match w.upgrade() {
        Some(x) => println!("up {}", x.borrow()),
        None => println!("gone"),
    }
    drop(shared);
    drop(s2);
    println!("{:?} {} {}", w.upgrade().is_none(), w.strong_count(), w.weak_count());
    let e: Weak<u8> = Weak::new();
    println!("{:?} {}", e.upgrade(), e.strong_count());
    let d = Rc::new(D(1));
    let wd = Rc::downgrade(&d);
    let wd2 = wd.clone();
    println!("{}", Rc::weak_count(&d));
    drop(d);
    println!("after rc drop {}", wd2.upgrade().is_none());
    drop(wd);
    drop(wd2);
    let x = Rc::new(7);
    let y = Rc::clone(&x);
    let z = Rc::new(7);
    println!("{} {}", Rc::ptr_eq(&x, &y), Rc::ptr_eq(&x, &z));
    let cs = vec![Cell::new(1), Cell::new(2)];
    cs[0].swap(&cs[1]);
    println!("{} {}", cs[0].get(), cs[1].get());
    let mut m = RefCell::new(4);
    *m.get_mut() += 1;
    *m.borrow_mut() *= 10;
    println!("{}", m.borrow());
}
