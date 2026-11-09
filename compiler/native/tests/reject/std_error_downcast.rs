// expect: L0104
// Valid Rust. `dyn Error` downcast needs a type id in the Error
// vtable (like `dyn Any` has); not built yet (F3). The trait,
// `source` chains, and `Box<dyn Error>` conversions work.
use std::error::Error;
use std::fmt;
#[derive(Debug)]
struct Boom;
impl fmt::Display for Boom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "boom") }
}
impl Error for Boom {}
fn main() {
    let b: Box<dyn Error> = Box::new(Boom);
    println!("{:?}", b.downcast_ref::<Boom>());
}
