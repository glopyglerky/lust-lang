// `Rc::try_unwrap` succeeds with weak refs left (they then fail to
// upgrade); `make_mut` moves the value out from under weak refs.
use std::rc::Rc;
fn main() {
    let a = Rc::new(String::from("x"));
    let w = Rc::downgrade(&a);
    let v = Rc::try_unwrap(a);
    println!("{:?} {}", v, w.upgrade().is_none());
    let mut b = Rc::new(vec![1, 2]);
    let wb = Rc::downgrade(&b);
    Rc::make_mut(&mut b).push(3);
    println!("{:?} {} {}", b, wb.upgrade().is_none(), Rc::weak_count(&b));
    let mut c = Rc::new(5);
    let c2 = Rc::clone(&c);
    *Rc::make_mut(&mut c) += 1;
    println!("{} {}", c, c2);
    let mut d = Rc::new(1);
    let wd = Rc::downgrade(&d);
    println!("{:?}", Rc::get_mut(&mut d).is_some());
    drop(wd);
    println!("{:?}", Rc::get_mut(&mut d).is_some());
}
