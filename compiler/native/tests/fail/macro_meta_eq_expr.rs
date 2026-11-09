// expect: L0806
// rustc: expected expression, found end of macro arguments
// `meta` takes `=` and a whole expression: `1 -` does not end the
// expression, so the parse fails.
macro_rules! m { ($m:meta -) => { () }; ($($t:tt)*) => { () }; }
fn main() { m!(a = 1 -); }
