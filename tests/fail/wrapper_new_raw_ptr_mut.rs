// expect: L0110
// rustc E0308: a shared `&u8` does not coerce to `*mut u8`, also as a wrapper argument.
use std::cell::Cell;

fn main() {
    let x: u8 = 1;
    let c: Cell<*mut u8> = Cell::new(&x);
    println!("{}", c.get().is_null());
}
