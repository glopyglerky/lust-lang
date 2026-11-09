// reject: the item fragment does not parse as an item, even though the template only stringifies it (ARCH 6.2). Lust code: L0806.
// rustc-error: expected identifier, found `1f`
macro_rules! m { ($i:item) => { stringify!($i) } }
fn main() { let _s: &str = m!(fn 1f() {}); }
