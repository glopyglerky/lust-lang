// Rc's Default builds the same header as Rc::new: weak_count is 0,
// a Weak upgrades while the Rc lives, and the drop frees it.
use std::rc::Rc;

#[derive(Default)]
struct S {
    r: Rc<u8>,
    v: Vec<Rc<String>>,
}

fn main() {
    let s = S::default();
    println!("{} {} {}", Rc::strong_count(&s.r), Rc::weak_count(&s.r), s.v.len());
    let w = Rc::downgrade(&s.r);
    println!("{} {:?}", Rc::weak_count(&s.r), w.upgrade());
    let r: Rc<String> = Default::default();
    println!("{:?} {}", r, Rc::weak_count(&r));
    drop(s);
    println!("{:?}", w.upgrade());
}
