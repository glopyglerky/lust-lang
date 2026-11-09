// reject: local ambiguity inside an arm that cannot fully match is still a hard error (ARCH 6.2). Lust code: L0805.
// rustc-error: local ambiguity when calling macro
macro_rules! m { ($($i:ident)* $j:ident ;) => { "a" }; ($($t:tt)*) => { "b" }; }
fn main() { let _ = m!(x); }
