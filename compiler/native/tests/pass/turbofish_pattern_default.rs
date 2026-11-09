// A pattern's turbofish with a default must match the value's type.
struct P<T, U = u8>(T, U);
fn main() { let p = P(1i32, 2u8); let P::<i32>(a, b) = p; println!("{} {}", a, b); }
