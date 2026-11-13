trait K { fn k(&self) -> usize; }
struct W<T>(T);
impl<T> K for W<T> { fn k(&self) -> usize where u8: Copy { 1 } }
fn d(x: &dyn K) -> usize { x.k() }
fn main() { println!("{}", d(&W(String::new()))); }
