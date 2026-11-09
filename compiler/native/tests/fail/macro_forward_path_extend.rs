// expect: L0806
// rustc: no rules expected `::`
// A forwarded `path` is a whole path: `::x` after it does not extend it.
macro_rules! m { ($e:expr) => { () }; }
macro_rules! o { ($p:path) => { m!($p::x) }; }
fn main() { o!(a); }
