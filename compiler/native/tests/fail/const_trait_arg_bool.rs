// expect: L0110
// rustc E0308: the trait param is a usize.
trait Tr<const N: usize> { fn n(&self) -> usize { N } }
struct S;
impl Tr<true> for S {}
fn main() { println!("{}", 1); }
