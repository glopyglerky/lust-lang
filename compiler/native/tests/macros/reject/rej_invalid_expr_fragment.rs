// reject: the expr fragment does not parse, even though the template only stringifies it (ARCH 6.2). Lust code: L0806.
// rustc-error: or an operator, found `2`
macro_rules! m { ($e:expr) => { stringify!($e) } }
fn main() { let _s: &str = m!((1 2, 3)); }
