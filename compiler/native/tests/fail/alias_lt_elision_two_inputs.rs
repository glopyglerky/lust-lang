// expect: L0202
// rustc E0106: two elided input lifetimes through the alias.

type R<'a> = &'a str;
fn f(x: R, y: R) -> R { x }
fn main() {}
