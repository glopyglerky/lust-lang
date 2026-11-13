// A bound `W: Tr<'x>` forwards to another generic fn's `V: Tr<'d>`: `'d` is `'x`.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) -> usize { w.list(data) }
fn outer<'x, W: Visit<'x>>(w: &mut W, d: &'x [u8]) -> usize { feed(w, d) + feed(w, d) }
struct C(usize);
impl<'d> Visit<'d> for C { fn list(&mut self, l: &'d [u8]) -> usize { self.0 += l.len(); self.0 } }
fn main() { let v = vec![1u8, 2]; let mut c = C(0); println!("{}", outer(&mut c, &v)); }
