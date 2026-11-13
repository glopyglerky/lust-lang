// expect: L0306
// Cell::swap makes both cells' T one.
use std::cell::Cell;
fn main() {
    let x = 5;
    let a = Cell::new(&x);
    {
        let y = 6;
        let b = Cell::new(&y);
        a.swap(&b);
    }
    println!("{}", a.get());
}
