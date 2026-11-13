// reject: the path fragment's generic arguments do not parse, even though the template only stringifies it (ARCH 6.2). Lust code: L0806.
// rustc-error: or an operator, found `2`
macro_rules! m { ($p:path) => { stringify!($p) } }
fn main() { let _s: &str = m!(a::<1 2>); }
