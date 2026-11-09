// A trait method through UFCS on a concrete type with `impl<'d> Tr<'d>`.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct C(usize);
impl<'d> Visit<'d> for C { fn list(&mut self, l: &'d [u8]) -> usize { self.0 += l.len(); self.0 } }
fn main() { let v = vec![1u8, 2]; let mut c = C(0); println!("{}", Visit::list(&mut c, &v)); }
