// expect: L0104
// rustc E0599: a `self: &Rc<Self>` method needs an `Rc` receiver.
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
    let n = N(3);
    println!("{}", n.visit());
}
