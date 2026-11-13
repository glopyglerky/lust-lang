const K: u32 = 10;
const fn f() -> u32 { K * 3 }
fn g() -> u32 { let x = const { f() }; x }
fn main() { println!("{}", g()); }
