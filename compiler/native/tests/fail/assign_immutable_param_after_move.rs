// expect: L0302
// rustc E0384: an immutable parameter, moved, then assigned.
fn f(a: String) { drop(a); a = String::new(); println!("{}", a); }
fn main() { f(String::new()); }
