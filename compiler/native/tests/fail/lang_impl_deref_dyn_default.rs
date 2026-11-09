// expect: L0110
// `type Target = dyn Tr` is `dyn Tr + 'static`, so `deref` must return
// `&(dyn Tr + 'static)`; `&dyn Tr` is `&'a (dyn Tr + 'a)` (rustc: the
// impl item signature does not match the trait's).
use std::ops::Deref;

trait Tr {
    fn hi(&self) -> u8;
}

struct A;

impl Tr for A {
    fn hi(&self) -> u8 {
        1
    }
}

struct H {
    b: Box<dyn Tr>,
}

impl Deref for H {
    type Target = dyn Tr;
    fn deref(&self) -> &dyn Tr {
        &*self.b
    }
}

fn main() {
    let h = H { b: Box::new(A) };
    println!("{}", h.hi());
}
