// expect: L0102
// A child module does not see its parent's imports.
use std::cell::*;
mod m {
    pub fn f() -> i32 {
        let c = Cell::new(1);
        c.get()
    }
}
fn main() {
    println!("{}", m::f());
}
