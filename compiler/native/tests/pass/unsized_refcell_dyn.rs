use std::cell::RefCell;
use std::rc::Rc;
trait W { fn write(&mut self, s: &str); fn get(&self) -> String; }
struct S(String);
impl W for S { fn write(&mut self, s: &str) { self.0.push_str(s); } fn get(&self) -> String { self.0.clone() } }
fn main() {
    let w: Rc<RefCell<dyn W>> = Rc::new(RefCell::new(S(String::new())));
    w.borrow_mut().write("hi");
    println!("{}", w.borrow().get());
}
