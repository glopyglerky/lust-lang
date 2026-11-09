// expect: L0102
// rustc E0412: an alias body is checked even when the alias is unused.

type R<'a> = &'a String;
type Bad = Missing;
fn main() { let s = String::from("x"); let r: R = &s; println!("{}", r); }
