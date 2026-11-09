// expect: L0733
// rustc: the result of `t.name()` through the bound borrows `t`, which
// is not `'static`.
trait Tr { fn name(&self) -> &str; }
fn nm<T: Tr>(t: &T) -> &'static str { t.name() }
fn main() {}
