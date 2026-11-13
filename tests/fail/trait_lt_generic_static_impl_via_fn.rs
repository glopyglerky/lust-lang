// expect: L0306
// rustc E0597: a generic call selects `impl Tr<'static>`.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct K(usize);
impl Visit<'static> for K { fn list(&mut self, l: &'static [u8]) -> usize { l.len() } }
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) -> usize { w.list(data) }
fn main() { let mut k = K(0); let v = vec![1u8]; println!("{}", feed(&mut k, &v)); }
