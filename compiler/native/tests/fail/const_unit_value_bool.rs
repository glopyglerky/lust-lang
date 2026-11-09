// expect: L0110
// rustc E0308: `true` for a usize param (lust read it as 1 before).
struct S<const N: usize>;
impl<const N: usize> S<N> { fn n(&self) -> usize { N } }
fn main() { println!("{}", S::<true>.n()); }
