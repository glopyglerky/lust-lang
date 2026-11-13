// reject: fragment parser hard failure: the expr parser began on `1 +` and failed, so the token arm never gets a try (ARCH 6.2). Lust code: L0806.
// rustc-error: expected expression, found end of macro arguments
macro_rules! m { ($e:expr ;) => { 1 }; (1 +) => { 2 } }
fn main() { let _ = m!(1 +); }
