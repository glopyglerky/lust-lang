// reject: fragment follow set: expr may only be followed by => , ; (ARCH 6.2).
// rustc-error: is followed by `+`, which is not allowed for `expr` fragments
macro_rules! m { ($e:expr +) => { $e } }
fn main() { let _ = m!(1 +); }
