// expect: L0733
// rustc: borrowed data escapes. Through the bound `T: Tr`, `put` takes
// `&'static str`; the template passes its own elided `&str`.
trait Tr { fn put(&mut self, s: &'static str); }
fn go<T: Tr>(t: &mut T, s: &str) { t.put(s); }
fn main() {}
