// expect: L0105
// rustc E0107: S takes one generic argument.
struct S<A> { a: A }
fn main() { let v = S::<u8, u16> { a: 5 }; println!("{}", v.a); }
