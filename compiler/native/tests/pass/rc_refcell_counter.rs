// Rc<RefCell<T>> fixture set (W-STD-B gate): a shared counter through clones, try_borrow_mut while borrowed, replace, take, and try_unwrap.
use std::cell::RefCell;
use std::rc::Rc;

fn bump(c: &Rc<RefCell<i32>>, by: i32) {
    *c.borrow_mut() += by;
}

fn main() {
    let a = Rc::new(RefCell::new(0));
    let b = Rc::clone(&a);
    let c = a.clone();
    bump(&a, 1);
    bump(&b, 10);
    *c.borrow_mut() *= 2;
    println!("{} {} {}", a.borrow(), Rc::strong_count(&a), Rc::ptr_eq(&a, &c));
    {
        let r1 = a.borrow();
        let r2 = b.borrow();
        println!("{} {}", *r1 + *r2, a.try_borrow_mut().is_err());
    }
    println!("{}", a.try_borrow_mut().is_ok());
    let old = a.replace(5);
    let taken = b.take();
    println!("{} {} {}", old, taken, c.borrow());
    drop(b);
    drop(c);
    println!("{}", Rc::strong_count(&a));
    let inner = Rc::try_unwrap(a).ok().unwrap().into_inner();
    println!("{}", inner);
}
