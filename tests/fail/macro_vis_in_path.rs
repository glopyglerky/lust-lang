// expect: L0806
// rustc: expected one of `)` or `::`, found `b`
// `pub(in` commits to a module path and a closing paren.
macro_rules! m { ($v:vis ($($t:tt)*)) => { () }; }
fn main() { m!(pub (in a b)); }
