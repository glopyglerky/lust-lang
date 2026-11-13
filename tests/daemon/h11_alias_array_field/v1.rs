type A = [u8; 2];
struct S { a: A }
fn main() { let s = S { a: [1, 2] }; println!("{} {}", s.a.len(), std::mem::size_of::<S>()); }
