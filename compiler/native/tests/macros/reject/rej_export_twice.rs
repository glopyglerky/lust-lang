// reject: two #[macro_export] definitions under one name (ARCH 6.1). Lust code: L0807.
// rustc-error: defined multiple times
#[macro_export]
macro_rules! m { () => { 1 }; }
#[macro_export]
macro_rules! m { () => { 2 }; }
fn main() { let _ = m!(); }
