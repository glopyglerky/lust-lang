// expect: L0733
// rustc: `IntoIter<Inv<'c>>` is not an `IntoIter<Inv<'b>>`: `Inv` is
// invariant, so `'b` must equal `'c`.
use std::cell::Cell;
struct Inv<'a>(Cell<&'a u8>);
fn k<'b, 'c: 'b>(c: Vec<Inv<'c>>) -> Option<Inv<'b>> {
    c.into_iter().next()
}
fn main() {}
