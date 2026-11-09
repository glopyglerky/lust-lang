// expect: L0805
// rustc: local ambiguity when calling macro `m`: multiple parsing options: built-in NTs vis ('v') or 1 other option.
// `vis` can begin on any identifier, so at `a` the optional group (a `vis`
// first) and the token `a` after it are both live: rustc stops there.
macro_rules! m { ($($v:vis x)? a) => { () }; }
fn main() { m!(a); }
