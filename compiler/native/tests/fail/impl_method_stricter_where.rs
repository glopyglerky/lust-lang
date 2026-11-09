// expect: L0110
// rustc E0276: impl has stricter requirements than trait.
trait K { fn k(&self) -> usize; }
fn d(x: &dyn K) -> usize { x.k() }
struct W<T>(T);
impl<T> K for W<T> { fn k(&self) -> usize where T: Copy { 1 } }
fn main() { println!("{}", d(&W(String::new()))); }
