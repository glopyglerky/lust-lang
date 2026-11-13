// reject: unknown macro name (ARCH 6.1). Lust code: L0802.
// rustc-error: cannot find macro `nosuch` in this scope
fn main() { nosuch!(); }
