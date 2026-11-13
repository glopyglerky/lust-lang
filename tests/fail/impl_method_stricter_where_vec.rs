// expect: L0110
// rustc E0277/E0276: the impl method adds `where Vec<T>: Clone`.
trait K { fn k(&self) -> usize; }
struct W<T>(T);
impl<T> K for W<T> { fn k(&self) -> usize where Vec<T>: Clone { 1 } }
fn main() { println!("{}", W(1u8).k()); }
