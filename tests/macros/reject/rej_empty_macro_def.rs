// reject: a macro must contain at least one rule (ARCH 6.2). Lust code: L0807.
// rustc-error: macros must contain at least one rule
macro_rules! m {}
fn main() {}
