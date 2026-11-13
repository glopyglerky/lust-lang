// A std glob followed through a `pub use` glob of a crate module.
mod a {
    pub use std::cell::*;
}
use a::*;
fn main() {
    let c = Cell::new(3);
    println!("{}", c.get());
}
