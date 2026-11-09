// Cell's Debug formats a copy (`self.get()`): a Debug impl that sets the
// cell does not change what that Debug call prints.
use std::cell::Cell;
use std::fmt;

#[derive(Clone, Copy)]
struct S<'a> {
    c: Option<&'a Cell<S<'a>>>,
    v: u8,
}

impl fmt::Debug for S<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = self.c {
            c.set(S { c: None, v: self.v + 10 });
        }
        write!(f, "S{}", self.v)
    }
}

fn main() {
    let cell = Cell::new(S { c: None, v: 1 });
    cell.set(S { c: Some(&cell), v: 2 });
    println!("{:?}", cell);
    println!("{:?}", cell);
    println!("{:#?}", Cell::new(S { c: None, v: 3 }));
}
