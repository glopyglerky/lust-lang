// An unused alias lifetime is fine; `fn f(x: R) -> R` elides as usual.
type Unused<'a> = i32;
type R<'a> = &'a str;
fn f(x: R) -> R { x }
fn main() { let u: Unused<'static> = 3; let s = String::from("x"); println!("{} {}", u, f(&s)); }
