// expect: L0733
// rustc E0621: `Vec<&'b Cell<&'b u8>>::push` needs the stored `Cell`'s
// lifetime equal to `'b` (Cell is invariant), not only longer.
use std::cell::Cell;
fn k<'b>(v: &mut Vec<&'b Cell<&'b u8>>, x: &'b Cell<&'_ u8>) {
    v.push(x);
}
fn main() {
    let a = 1u8;
    let c = Cell::new(&a);
    let mut v = Vec::new();
    k(&mut v, &c);
    println!("{}", v.len());
}
