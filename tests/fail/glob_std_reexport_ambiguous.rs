// expect: L0507
// rustc E0659: two globs bring different items named `Rc` (a std re-export
// and a crate struct), and the name is used.
mod a {
    pub use std::rc::Rc;
}
mod b {
    pub struct Rc(pub u8);
}
use a::*;
use b::*;
fn main() {
    let r = Rc::new(1u8);
    println!("{}", r);
}
