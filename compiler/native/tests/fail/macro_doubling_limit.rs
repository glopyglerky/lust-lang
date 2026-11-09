// expect: L0801
// rustc: recursion limit reached while expanding `count!`. Each level doubles the input;
// lust took 8 s and 12 GB here before repetition matching was linear.
#![recursion_limit = "16"]
macro_rules! count { () => { 0 }; ($h:tt $($t:tt)*) => { 1 + count!($(pub$t)*) }; }
fn main() { println!("{}", count!(a b c)); }
