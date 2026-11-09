// A value that holds a shared borrow of itself (through a `Cell`) can still
// be read: the borrow is shared, so reads do not conflict with it.
use std::cell::Cell;
struct C<'a> {
    c: Cell<Option<&'a C<'a>>>,
    n: u8,
}
fn main() {
    let c = C { c: Cell::new(None), n: 3 };
    c.c.set(Some(&c));
    println!("{}", c.c.get().unwrap().n);
    println!("{}", c.n);
}
