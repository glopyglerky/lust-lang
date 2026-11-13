// expect: L0110
// rustc E0107: wrong count of trait lifetime arguments.
trait Visit<'d> { fn n(&self) -> usize; }
struct K;
impl<'a, 'b> Visit<'a, 'b> for K { fn n(&self) -> usize { 0 } }
fn main() {}
