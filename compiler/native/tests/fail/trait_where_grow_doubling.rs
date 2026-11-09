// expect: L0611
// rustc E0275: the impl's where clause asks G<(T, T)>: Tr, so proving G<u8>: Tr never ends.
// lust hung here: its unify occurs check and error scan walked the doubling type as a tree.
trait Tr { fn go(&self) -> u32; }
struct G<T>(T);
impl<T> Tr for G<T> where G<(T, T)>: Tr { fn go(&self) -> u32 { 1 } }
fn main() { println!("{}", G(1u8).go()); }
