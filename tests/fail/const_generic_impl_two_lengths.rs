// expect: L0104
// rustc E0599: one N cannot be both 2 and 3.
trait Pair { fn n(&self) -> usize; }
impl<const N: usize> Pair for ([u8; N], [u16; N]) { fn n(&self) -> usize { N } }
fn main() { let p = ([1u8, 2], [3u16, 4, 5]); println!("{}", p.n()); }
