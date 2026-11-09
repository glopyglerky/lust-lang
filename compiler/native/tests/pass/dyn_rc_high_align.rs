// Rc<dyn Tr>, Arc<dyn Tr>, their Weaks, and Rc<RefCell<dyn Tr>> of a value
// aligned past 16: the counts start the allocation and the value sits at its
// alignment (RcInner's layout), so the counts are found from the vtable.
use std::cell::RefCell;
use std::rc::{self, Rc};
use std::sync::{self, Arc};
trait Tr {
    fn v(&self) -> u32;
    fn bump(&mut self) {}
}
#[repr(align(64))]
struct Big(u32);
impl Tr for Big {
    fn v(&self) -> u32 {
        self.0
    }
    fn bump(&mut self) {
        self.0 += 1;
    }
}
impl Drop for Big {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
#[repr(align(32))]
struct Mid(u8);
impl Tr for Mid {
    fn v(&self) -> u32 {
        self.0 as u32
    }
}
fn main() {
    let a: Arc<dyn Tr> = Arc::new(Big(7));
    let b = a.clone();
    let wa: sync::Weak<dyn Tr> = Arc::downgrade(&a);
    println!("{} {} {}", b.v(), Arc::strong_count(&a), Arc::weak_count(&a));
    drop(a);
    drop(b);
    println!("{}", wa.upgrade().is_none());
    let r: Rc<dyn Tr> = Rc::new(Mid(8));
    let s = r.clone();
    let wr: rc::Weak<dyn Tr> = Rc::downgrade(&r);
    println!("{} {} {} {}", s.v(), Rc::strong_count(&r), Rc::weak_count(&r), wr.strong_count());
    drop(r);
    println!("{}", wr.upgrade().map_or(0, |x| x.v()));
    drop(s);
    let c: Rc<RefCell<dyn Tr>> = Rc::new(RefCell::new(Big(1)));
    let d = c.clone();
    c.borrow_mut().bump();
    println!("{} {}", d.borrow().v(), Rc::strong_count(&c));
}
