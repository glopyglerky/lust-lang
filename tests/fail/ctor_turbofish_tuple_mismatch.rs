// expect: L0110
// rustc E0308: the turbofish fixes A = u8.
struct W<A>(A, usize);
fn main() { let v = W::<u8>(5u16, 2); println!("{}", v.0); }
