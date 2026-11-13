// expect: L0110
// rustc E0308: the pattern's default U = u8 does not match u16.
struct P<T, U = u8>(T, U);
fn main() { let p = P(1i32, 2u16); let P::<i32>(a, b) = p; println!("{} {}", a, b); }
