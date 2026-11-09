// reject: opaque fragments: a forwarded expr cannot match a literal arm of the inner macro (ARCH 6.2).
// rustc-error: no rules expected `expr` metavariable
macro_rules! inner { (3) => { 3 } }
macro_rules! outer { ($x:expr) => { inner!($x) } }
fn main() { let _ = outer!(3); }
