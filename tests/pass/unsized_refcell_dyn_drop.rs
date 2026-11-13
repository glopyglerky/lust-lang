use std::cell::RefCell; use std::rc::Rc; use std::sync::Arc;
trait T { fn v(&self) -> u32; fn bump(&mut self); }
struct D(u32);
impl T for D { fn v(&self) -> u32 { self.0 } fn bump(&mut self) { self.0 += 1; } }
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
struct Big { a: u128, b: String }
impl T for Big { fn v(&self) -> u32 { self.a as u32 + self.b.len() as u32 } fn bump(&mut self) { self.a += 1; } }
fn main() {
    let a: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(D(1)));
    let b = a.clone();
    b.borrow_mut().bump();
    drop(a);
    println!("v {}", b.borrow().v());
    drop(b);
    let c: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(Big { a: 1 << 70, b: String::from("xy") }));
    c.borrow_mut().bump();
    println!("big {}", c.borrow().v());
    let bx: Box<RefCell<dyn T>> = Box::new(RefCell::new(D(7)));
    bx.borrow_mut().bump();
    println!("bx {}", bx.borrow().v());
    let ar: Arc<RefCell<dyn T>> = Arc::new(RefCell::new(D(9)));
    ar.borrow_mut().bump();
    println!("ar {}", ar.borrow().v());
    println!("end");
}
