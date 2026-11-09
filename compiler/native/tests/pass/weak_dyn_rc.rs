// rc::Weak<dyn Tr>: downgrade, upgrade, counts, ptr_eq, Debug, Weak<T> to Weak<dyn Tr> (also dangling), drop order.
use std::rc::{Rc, Weak};
use std::fmt::Debug;
trait Show { fn show(&self) -> String; }
#[derive(Debug)]
struct A(u8);
impl Show for A { fn show(&self) -> String { format!("A{}", self.0) } }
impl Drop for A { fn drop(&mut self) { println!("drop A{}", self.0); } }
#[repr(align(32))]
struct Big(u64);
impl Show for Big { fn show(&self) -> String { format!("Big{}", self.0) } }
impl Drop for Big { fn drop(&mut self) { println!("drop Big{}", self.0); } }
struct Node { kids: Vec<Weak<dyn Show>> }
fn main() {
    let r: Rc<dyn Show> = Rc::new(A(1));
    let w = Rc::downgrade(&r);
    println!("{} {} {}", w.upgrade().is_some(), w.strong_count(), w.weak_count());
    let w2 = w.clone();
    println!("{} {:?}", Weak::ptr_eq(&w, &w2), w2);
    if let Some(x) = w2.upgrade() { println!("{}", x.show()); }
    drop(r);
    println!("{} {} {}", w.upgrade().is_none(), w.strong_count(), w.weak_count());
    let d: Weak<dyn Show> = Weak::<A>::new();
    println!("{} {} {}", d.upgrade().is_none(), d.strong_count(), d.weak_count());
    let b = Rc::new(Big(9));
    let wb = Rc::downgrade(&b);
    let wb: Weak<dyn Show> = wb;
    let n = Node { kids: vec![wb.clone(), w2.clone(), d.clone()] };
    for k in &n.kids { match k.upgrade() { Some(x) => println!("{}", x.show()), None => println!("gone") } }
    println!("{} {}", Rc::strong_count(&b), Rc::weak_count(&b));
    let rb: Rc<dyn Show> = b;
    println!("{} {}", Rc::strong_count(&rb), Rc::weak_count(&rb));
    drop(rb);
    println!("{}", wb.upgrade().is_none());
    drop(n);
    let _ = Rc::new(A(3)) as Rc<dyn Show>;
}
