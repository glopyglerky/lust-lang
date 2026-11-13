// A trait lifetime bound in a where clause, and an impl where clause on lifetimes.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct K(usize);
impl<'d> Visit<'d> for K where 'd: 'd { fn list(&mut self, l: &'d [u8]) -> usize { l.len() } }
fn feed<'d, V>(w: &mut V, data: &'d [u8]) -> usize where V: Visit<'d> { w.list(data) }
fn main() { let mut k = K(0); let v = vec![1u8]; println!("{}", feed(&mut k, &v)); }
