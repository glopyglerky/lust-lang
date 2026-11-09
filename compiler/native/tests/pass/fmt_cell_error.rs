// panic: a formatting trait implementation returned an error when the underlying stream did not
// A Debug error inside a std cell or a boxed slice reaches format!.
use std::cell::{Cell, RefCell};
use std::fmt;

#[derive(Clone, Copy)]
struct A(u8);

impl fmt::Debug for A {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 { Err(fmt::Error) } else { write!(f, "A{}", self.0) }
    }
}

fn main() {
    let b: Box<[A]> = vec![A(1), A(2)].into_boxed_slice();
    println!("{:?}", b);
    let c = Cell::new(A(3));
    println!("{:?}", c);
    let r = RefCell::new(A(4));
    println!("{:?} {:?}", r, r.borrow());
    let z = RefCell::new(A(0));
    let s = format!("{:?}", z);
    println!("unreachable {}", s);
}
