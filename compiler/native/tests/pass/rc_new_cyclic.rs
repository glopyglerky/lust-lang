// Rc::new_cyclic: data_fn sees a Weak with strong count 0 (upgrade fails),
// the result has counts 1/1, and drops run as std's.
use std::rc::{Rc, Weak};
use std::cell::RefCell;
struct Node { me: Weak<Node>, val: u32, kids: RefCell<Vec<Rc<Node>>> }
impl Drop for Node { fn drop(&mut self) { println!("drop node {}", self.val); } }
struct MapEntry { this: Weak<MapEntry>, k: String }
fn new_map(k: &str) -> Rc<MapEntry> {
    Rc::new_cyclic(|this| MapEntry { this: this.clone(), k: k.to_string() })
}
fn main() {
    let n = Rc::new_cyclic(|w: &Weak<Node>| {
        println!("in: up={} sc={} wc={}", w.upgrade().is_none(), w.strong_count(), w.weak_count());
        Node { me: w.clone(), val: 7, kids: RefCell::new(vec![]) }
    });
    println!("{} {} {}", Rc::strong_count(&n), Rc::weak_count(&n), n.me.upgrade().map(|x| x.val).unwrap_or(0));
    let c = n.me.upgrade().unwrap();
    println!("{} {}", Rc::strong_count(&n), Rc::ptr_eq(&n, &c));
    drop(c);
    let s = String::from("cap");
    let m = Rc::new_cyclic(move |w| { let _ = &s; Node { me: w.clone(), val: s.len() as u32, kids: RefCell::new(vec![]) } });
    m.kids.borrow_mut().push(Rc::new_cyclic(|w| Node { me: w.clone(), val: 99, kids: RefCell::new(vec![]) }));
    let e = new_map("k1");
    println!("{} {}", e.k, e.this.upgrade().unwrap().k);
    let w = Rc::downgrade(&n);
    drop(n);
    println!("after drop {}", w.upgrade().is_none());
    let unit: Rc<u8> = Rc::new_cyclic(|_| 5);
    println!("{}", unit);
}
