// expect: L0306
// Cell::set stores through a shared reference.
use std::cell::Cell;
fn main() {
    let c: Cell<&i32>;
    let x = 5;
    c = Cell::new(&x);
    {
        let y = 6;
        c.set(&y);
    }
    println!("{}", c.get());
}
