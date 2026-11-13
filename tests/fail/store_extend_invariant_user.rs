// expect: L0733
// rustc E0521: `extend`, `insert` and `push_back` store a value of an
// invariant user type; its lifetime must equal the container's.
use std::cell::Cell;
use std::collections::VecDeque;
struct Inv<'a>(Cell<&'a u8>);
fn k<'b, 'c: 'b>(v: &mut VecDeque<Inv<'b>>, x: Inv<'c>) {
    v.extend([x]);
}
fn main() {
    let a = 1u8;
    let mut v = VecDeque::new();
    k(&mut v, Inv(Cell::new(&a)));
    println!("{}", v.len());
}
