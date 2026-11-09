const fn k() -> u32 { 30 }
fn f() -> u32 { let x = const { k() }; x + 1 }
fn main() { println!("{}", f()); }
