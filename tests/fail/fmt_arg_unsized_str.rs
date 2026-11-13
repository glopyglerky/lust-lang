// expect: L0110
// `*s` for a `String` is a `str`, not `Sized` (rustc E0277).
fn main() { let s = String::from("a"); let t = format!("{}", *s); println!("{}", t); }
