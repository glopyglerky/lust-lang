// expect: L0104
// rustc E0277: with no bound, `T: Into<String>` is unknown.
fn g<T>(x: T) -> String { x.into() }
fn main() { println!("{}", g("x")); }
