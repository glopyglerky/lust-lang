// expect: L0631
// rustc E0658: `Cell::set` is only conditionally const (it drops the old
// value), so a const fn cannot call it.
use std::cell::Cell;
const fn f(c: &Cell<u8>) {
    c.set(3)
}
fn main() {
    let c = Cell::new(1);
    f(&c);
    println!("{}", c.get());
}
