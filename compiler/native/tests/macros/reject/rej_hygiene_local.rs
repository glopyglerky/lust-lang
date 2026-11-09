// reject: hygiene: a local the macro introduces is not accessible to the caller (ARCH 6.3).
// rustc-error: cannot find value `secret` in this scope
macro_rules! m { () => { let secret = 3; } }
fn main() { m!(); println!("{}", secret); }
