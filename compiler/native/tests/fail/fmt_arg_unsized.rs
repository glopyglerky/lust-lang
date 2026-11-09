// expect: L0110
// Format arguments must be `Sized` (rustc E0277): `*v` is a `[u32]`.
fn main() { let v = vec![1u32]; println!("{:?}", *v); }
