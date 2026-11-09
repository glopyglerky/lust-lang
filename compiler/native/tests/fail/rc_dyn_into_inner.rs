// expect: L0641
// Rc::into_inner needs a sized T (rustc E0277); lust rejects it too.
use std::rc::Rc;

trait Show {
    fn show(&self) -> u8;
}

struct A(u8);

impl Show for A {
    fn show(&self) -> u8 {
        self.0
    }
}

fn main() {
    let r: Rc<dyn Show> = Rc::new(A(1));
    let v = Rc::into_inner(r);
    println!("{}", v.is_some());
}
