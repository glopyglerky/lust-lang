const K: u32 = 1;
const fn f() -> u32 { K + 1 }
fn g() -> u32 { let x = const { f() }; x }
fn main() { println!("{}", g()); }
