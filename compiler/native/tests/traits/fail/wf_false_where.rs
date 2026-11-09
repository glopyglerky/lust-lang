// rustc-error: E0277
// lust-expect: L0612
fn f() -> u8 where String: Copy { 1 }
fn main() { println!("{}", f()); }
