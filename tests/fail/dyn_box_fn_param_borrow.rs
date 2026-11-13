// expect: L0306
// rustc E0597: a `Box<dyn Fn>` param is `'static`, the closure borrows `s`.
fn keep(f: Box<dyn Fn() -> usize>) -> usize { f() }
fn main() { let s = String::from("xyz"); println!("{}", keep(Box::new(|| s.len()))); }
