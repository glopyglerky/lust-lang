// expect: L0733
// rustc: lifetime may not live long enough. The `map` closure puts `r` into
// both parts of the stored pair, so `'z` must outlive `'a` and `'b`.
use std::cell::Cell;
fn k<'a, 'b, 'z>(c: &mut Vec<(&'a u8, Cell<&'b u8>)>, v: Vec<&'z u8>) {
    c.extend(v.into_iter().map(|r| (r, Cell::new(r))));
}
fn main() {}
