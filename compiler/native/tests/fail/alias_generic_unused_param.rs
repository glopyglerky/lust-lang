// expect: L0110
// rustc E0091: an alias's type param must be used.
type X<T> = u32;
fn main() { let x: X<String> = 3; println!("{x}"); }
