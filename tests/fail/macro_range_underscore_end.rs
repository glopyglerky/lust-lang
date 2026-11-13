// expect: L0806
// rustc: no rules expected reserved identifier `_`
// `_` does not begin a range end: the expression is `..` alone.
macro_rules! m { ($e:expr => x) => { () }; }
fn main() { m!(.. _ => x); }
