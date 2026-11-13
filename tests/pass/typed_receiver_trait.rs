// Typed receivers on trait methods (`self: &Rc<Self>`, `Rc<Self>`,
// `Box<Self>`): the call matches the one receiver variant the method
// names (it was ambiguous with itself, L0720), through references too.
use std::rc::Rc;

trait Visit {
    fn visit(self: &Rc<Self>) -> u8;
    fn take(self: Rc<Self>) -> u8;
    fn boxed(self: Box<Self>) -> u8;
    fn default_visit(self: &Rc<Self>) -> u8 {
        7
    }
}

struct N(u8);

impl Visit for N {
    fn visit(self: &Rc<Self>) -> u8 {
        self.0 + Rc::strong_count(self) as u8
    }
    fn take(self: Rc<Self>) -> u8 {
        self.0 * 2
    }
    fn boxed(self: Box<Self>) -> u8 {
        self.0 * 3
    }
}

fn main() {
    let n = Rc::new(N(3));
    let m = n.clone();
    let r = &n;
    let rr = &r;
    println!("{} {} {} {}", n.visit(), r.visit(), rr.visit(), n.default_visit());
    println!("{} {}", m.take(), Box::new(N(5)).boxed());
}
