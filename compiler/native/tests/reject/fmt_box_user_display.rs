// expect: L0641
// rustc accepts and prints "boxed"; fmt glue formats a Box through its
// contents, so lust rejects the impl rather than skip it (it aborted).
use std::fmt;
struct My;
impl fmt::Display for Box<My> { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "boxed") } }
fn main() { println!("{}", Box::new(My)); }
