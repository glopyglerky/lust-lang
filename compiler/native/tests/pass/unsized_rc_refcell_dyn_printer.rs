use std::cell::RefCell;
use std::rc::Rc;
trait Out { fn put(&mut self, s: &str); fn text(&self) -> String; fn n(&self) -> usize { self.text().len() } }
struct Buf { s: String, writes: u32 }
impl Out for Buf { fn put(&mut self, s: &str) { self.s.push_str(s); self.writes += 1; } fn text(&self) -> String { format!("{}#{}", self.s, self.writes) } }
struct Printer { w: Rc<RefCell<dyn Out>> }
impl Printer { fn emit(&self, s: &str) { self.w.borrow_mut().put(s); } }
fn main() {
    let shared: Rc<RefCell<dyn Out>> = Rc::new(RefCell::new(Buf { s: String::new(), writes: 0 }));
    let p1 = Printer { w: shared.clone() };
    let p2 = Printer { w: Rc::clone(&shared) };
    p1.emit("a"); p2.emit("b"); p1.emit("c");
    println!("{} {} {}", shared.borrow().text(), shared.borrow().n(), Rc::strong_count(&shared));
    let r1 = shared.borrow(); let r2 = shared.borrow();
    println!("{} {}", r1.text(), r2.n());
}
