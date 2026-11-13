// expect: L0734
// rustc E0308: the impl demands `'static` where the trait takes its `'d`.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct K(usize);
impl<'d> Visit<'d> for K { fn list(&mut self, l: &'static [u8]) -> usize { l.len() } }
fn main() {}
