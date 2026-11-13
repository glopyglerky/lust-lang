// expect: L0805
// rustc: local ambiguity when calling macro `m`: multiple parsing options: built-in NTs ident ('x').
// Two matcher paths (either optional `a` took the input) reach the same
// metavariable at `z`: two positions, so rustc reports an ambiguity.
macro_rules! m { ($(a)? $(a)? ; $x:ident) => { () }; }
fn main() { m!(a ; z); }
