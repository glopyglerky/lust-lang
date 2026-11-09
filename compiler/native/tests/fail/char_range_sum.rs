// expect: L0612
// rustc E0277: char is not Sum.
fn main() { let s: char = ('a'..'c').sum(); println!("{s}"); }
