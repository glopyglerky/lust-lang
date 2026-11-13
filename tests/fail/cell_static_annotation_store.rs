// expect: L0306
// A 'static annotation on a Cell's element cannot take a local borrow.
use std::cell::Cell;
static S: i32 = 7;
fn main() {
    let c: Cell<&'static i32> = Cell::new(&S);
    {
        let local = 5;
        let r: &Cell<&i32> = &c;
        r.set(&local);
    }
    println!("{}", c.get());
}
