// expect: L0306
// rustc E0597: a plain alias of `fn(&'static str)` keeps its `'static`
// input, so a borrow of a local cannot be passed (W-BORROW3: the alias
// erased it before).
type F = fn(&'static str);
fn h(s: &str) { println!("{}", s) }
fn main() { let f: F = h; let s = String::from("q"); f(&s); }
