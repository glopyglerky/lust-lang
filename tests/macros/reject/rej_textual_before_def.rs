// reject: textual scope: the call is before the definition point (ARCH 6.1). Lust code: L0802.
// rustc-error: cannot find macro `m` in this scope
fn main() { let _ = m!(); }
macro_rules! m { () => { 1 } }
