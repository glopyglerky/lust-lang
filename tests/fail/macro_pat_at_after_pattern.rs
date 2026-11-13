// expect: L0806
// rustc: pattern on wrong side of `@`
// rustc reads `@` after any pattern; only a binding may stand left of it.
macro_rules! m { ($p:pat => x) => { 1 }; ($($t:tt)*) => { 2 }; }
fn main() { let _ = m!(Some(x) @ y); }
