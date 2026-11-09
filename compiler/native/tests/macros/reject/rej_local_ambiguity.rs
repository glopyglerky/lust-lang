// reject: local ambiguity: two fragment parses can consume the same token (ARCH 6.2). Lust code: L0805.
// rustc-error: local ambiguity when calling macro
macro_rules! m { ($($i:ident)* $j:ident) => { () } }
fn main() { m!(x); }
