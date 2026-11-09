// expect: L0110
// rustc E0308: `fn(&'a str)` is not a `fn(&str)`, which takes any lifetime.
fn g<'a>(f: fn(&'a str)) -> fn(&str) { f }
fn main() { let _ = g; }
