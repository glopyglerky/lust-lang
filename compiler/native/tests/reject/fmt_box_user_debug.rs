// expect: L0641
// rustc accepts and prints "dbg-boxed"; lust printed the inner type.
use std::fmt;
struct My;
impl fmt::Debug for Box<My> { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "dbg-boxed") } }
fn main() { println!("{:?}", Box::new(My)); }
