// expect: L0806
// rustc: no rules expected `expr` metavariable
// A forwarded expression is one unit: `1 $x (a)` is not `1 - 1(a)`.
macro_rules! m { ($e:expr ,) => { () }; }
macro_rules! o { ($x:expr) => { m!(1 $x (a) ,) }; }
fn main() { o!(-1); }
