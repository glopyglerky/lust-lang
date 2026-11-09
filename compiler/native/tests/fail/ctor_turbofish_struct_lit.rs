// expect: L0110
// rustc E0308: the struct literal turbofish fixes A = u8.
struct S<A> { a: A }
fn main() { let v = S::<u8> { a: 5u16 }; println!("{}", v.a); }
