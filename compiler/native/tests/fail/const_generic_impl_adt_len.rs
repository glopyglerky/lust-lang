// expect: L0104
// rustc E0599: only Buf<4> has the impl.
struct Buf<const N: usize> { d: [u8; N] }
trait Cap { fn cap(&self) -> usize; }
impl Cap for Buf<4> { fn cap(&self) -> usize { 4 } }
fn main() { let b = Buf { d: [0u8; 6] }; println!("{}", b.cap()); }
