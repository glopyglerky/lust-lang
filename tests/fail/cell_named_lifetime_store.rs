// expect: L0306
// Cell is invariant: a store through a named lifetime ties the cell.
use std::cell::Cell;
fn put<'a>(c: &Cell<&'a i32>, x: &'a i32) {
    c.set(x);
}
static S: i32 = 7;
fn main() {
    let c: Cell<&'static i32> = Cell::new(&S);
    {
        let local = 5;
        put(&c, &local);
    }
    println!("{}", c.get());
}
