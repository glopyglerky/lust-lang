// reject: local ambiguity between a fragment and a plain token at the same input position (ARCH 6.2). Lust code: L0805.
// rustc-error: local ambiguity when calling macro
macro_rules! m { ($($i:ident)* x) => { 1 } }
fn main() { let _ = m!(x); }
