type A = [u8; 3];
struct S { a: A }
fn main() { let s = S { a: [1, 2, 3] }; println!("{} {}", s.a.len(), std::mem::size_of::<S>()); }
