struct S { a: u64 }
fn main() { println!("{} {}", std::mem::size_of::<[S; 2]>(), std::mem::size_of::<(S, u8)>()); }
