// expect: L0301
// rustc E0382: the String moves into the `impl Into<String>` argument.
fn own(s: impl Into<String>) -> String { s.into() }
fn main() { let s = String::from("a"); let t = own(s); println!("{} {}", t, s); }
