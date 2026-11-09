// expect: L0001
// rustc E0658: an `expr` fragment with an attribute that lands as an operand.
macro_rules! m { ($e:expr) => { 1 + $e } }
fn main() { println!("{}", m!(#[cfg(all())] 1)); }
