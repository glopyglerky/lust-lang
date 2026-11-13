// expect: L0806
// rustc: no rules expected `vis` metavariable
// An empty forwarded `vis` still stands between the pattern and `=`.
macro_rules! m { ($p:pat =) => { () }; }
macro_rules! o { ($x:vis a) => { m!([a, ..] $x =) }; }
fn main() { o!(a); }
