// expect: L0104
// rustc E0599: a `Box<N>` is not an `Rc<N>` receiver.
use std::rc::Rc;

trait Visit {
    fn visit(self: &Rc<Self>) -> u8;
}

struct N(u8);

impl Visit for N {
    fn visit(self: &Rc<Self>) -> u8 {
        self.0
    }
}

fn main() {
    let n = Box::new(N(3));
    println!("{}", n.visit());
}
