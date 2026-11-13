// expect: L0110
// rustc: `String` is forbidden as the type of a const generic parameter.
struct S<const N: String>;
fn main() { println!("{}", 1); }
