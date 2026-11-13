struct S { a: u8 }
fn f() -> u64 { let s = S { a: 3 }; s.a as u64 + std::mem::size_of::<S>() as u64 }
fn main() { println!("{}", f()); }
