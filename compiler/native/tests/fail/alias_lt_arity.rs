// expect: L0110
// rustc E0107: the alias takes one lifetime argument.

type R<'a> = &'a str;
fn f(x: R<'_, '_>) {}
fn main() {}
