// expect: L0805
// rustc: ambiguity: multiple successful parses
// Either optional `a` can take the input; two full matches are an error,
// even with no metavariable to bind.
macro_rules! m { ($(a)? $(a)?) => { () }; }
fn main() { m!(a); }
