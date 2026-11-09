// expect: L0306
// rustc E0597: `impl Tr<'static>` methods take `'static` data.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct K(usize);
impl Visit<'static> for K { fn list(&mut self, l: &'static [u8]) -> usize { l.len() } }
fn main() { let mut k = K(0); let v = vec![1u8]; println!("{}", k.list(&v)); }
