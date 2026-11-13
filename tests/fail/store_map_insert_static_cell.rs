// expect: L0733
// rustc E0521: a `Cell<&'static u8>` map value cannot hold a `'b` borrow.
use std::cell::Cell;
use std::collections::HashMap;
fn k<'b>(m: &mut HashMap<u8, Cell<&'static u8>>, x: Cell<&'b u8>) {
    m.insert(0, x);
}
fn main() {
    let mut m = HashMap::new();
    k(&mut m, Cell::new(&1));
    println!("{}", m.len());
}
