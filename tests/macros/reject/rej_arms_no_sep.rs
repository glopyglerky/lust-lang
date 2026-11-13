// reject: macro rules need `;` between them (ARCH 6.2). Lust code: L0807.
// rustc-error: expected `;`, found `(`
macro_rules! m { (a) => { 1 } (b) => { 2 } }
fn main() { let _ = m!(b); }
