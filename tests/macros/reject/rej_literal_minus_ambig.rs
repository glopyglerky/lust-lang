// reject: `-` can begin a literal, so the literal fragment and the `-` token are both live (ARCH 6.2). Lust code: L0805.
// rustc-error: local ambiguity when calling macro
macro_rules! m { ($($l:literal)? - x) => { "ok" }; }
fn main() { let _ = m!(- x); }
