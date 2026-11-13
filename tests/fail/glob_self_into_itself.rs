// expect: L0506
// rustc E0432: cannot glob-import a module into itself.
mod m {
    pub use std::cell::Cell;
    pub use self::*;
}
use m::Cell;
fn main() {
    let c = Cell::new(1);
    println!("{}", c.get());
}
