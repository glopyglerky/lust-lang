// expect: L0306
// Cell is invariant: a 'static cell cannot take a local borrow.
use std::cell::Cell;
static S: i32 = 7;
fn run(c: &Cell<&'static i32>) {
    let local = 5;
    let r: &Cell<&i32> = c;
    r.set(&local);
}
fn main() {
    let c = Cell::new(&S);
    run(&c);
    println!("{}", c.get());
}
