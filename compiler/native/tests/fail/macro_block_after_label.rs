// expect: L0806
// rustc: cannot use a `block` macro fragment here
// A forwarded `block` cannot follow a label inside a statement.
macro_rules! m { ($s:stmt , x) => { () }; ($($t:tt)*) => { () }; }
macro_rules! o { ($b:block) => { m!('l : $b , x) }; }
fn main() { o!({ 1 }); }
