// expect: L0110
// rustc E0599: `Box::new` needs a sized `T`, and the alias names `Box<dyn Fn()>`.
type BD = Box<dyn Fn()>;
fn main() { let b: BD = BD::new(|| ()); b(); }
