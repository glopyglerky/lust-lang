trait K { fn k(&self) -> usize; }
struct W<T>(T);
impl<T: Copy> K for W<T> { fn k(&self) -> usize where T: Copy { 1 } }
fn main() { println!("{}", W(1u8).k()); }
