// expect: L0807
// rustc: expected `(` or `{`, found `[`
// rustc checks `$` in every transcriber when the macro is defined.
macro_rules! m { () => { $[x] }; (a) => { 1 }; }
fn main() { let _ = m!(a); }
