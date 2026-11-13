// expect: L0733
// rustc E0521: `Cell<&'b u8>` is invariant, so the extended item
// `Cell<&'static u8>` must have exactly 'b (Vec::extend stores at the
// element's variance).
use std::cell::Cell;
fn k<'b>(c: &mut Vec<Cell<&'b u8>>, x: Cell<&'static u8>) { c.extend(std::iter::once(x)); }
fn main() { static A: u8 = 1; let mut v = vec![]; k(&mut v, Cell::new(&A)); println!("{}", v.len()); }
