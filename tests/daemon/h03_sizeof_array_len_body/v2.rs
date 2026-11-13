struct S { a: u32 }
fn f() -> usize { let a = [0u8; std::mem::size_of::<S>()]; a.len() }
fn main() { println!("{}", f()); }
