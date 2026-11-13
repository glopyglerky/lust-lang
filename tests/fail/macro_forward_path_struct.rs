// expect: L0806
// rustc: no rules expected `{`
// A forwarded `path` before a brace group is not a struct literal.
macro_rules! m { ($e:expr) => { () }; }
macro_rules! o { ($p:path) => { m!($p { a: 1 }) }; }
fn main() { o!(S); }
