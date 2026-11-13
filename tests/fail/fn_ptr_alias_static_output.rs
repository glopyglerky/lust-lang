// expect: L0110
// rustc E0308: a fn that returns a borrow of its argument does not become
// a plain alias of `fn(&str) -> &'static str` (W-BORROW3: the alias erased
// the `'static` before).
type F = fn(&str) -> &'static str;
fn h(s: &str) -> &str { s }
fn main() { let f: F = h; println!("{}", f("x")); }
