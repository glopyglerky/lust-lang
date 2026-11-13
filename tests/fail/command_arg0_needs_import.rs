// expect: L0104
// rustc E0599: arg0 needs `use std::os::unix::process::CommandExt`.
fn main() { let mut c = std::process::Command::new("true"); c.arg0("x"); }
