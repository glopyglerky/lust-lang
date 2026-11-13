// expect: L0806
// rustc: unexpected generic arguments in path
// A forwarded `path` read as a `meta` is a module path: no generic args.
macro_rules! m { ($m:meta) => { () }; }
macro_rules! o { ($p:path) => { m!($p) }; }
fn main() { o!(Vec::<u8>::new); }
