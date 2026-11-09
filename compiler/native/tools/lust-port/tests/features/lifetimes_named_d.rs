// feature: lifetimes.named
// form: a trait with a lifetime param, as in ts-rust ast/node.rs ChildVisit<'d>: impls for 'static and for all 'd, a generic bound
trait Visit<'d> {
    fn list(&mut self, l: &'d [u8]) -> usize;
}
struct Count(usize);
impl<'d> Visit<'d> for Count {
    fn list(&mut self, l: &'d [u8]) -> usize { self.0 += l.len(); self.0 }
}
struct Keep(Vec<&'static [u8]>);
impl Visit<'static> for Keep {
    fn list(&mut self, l: &'static [u8]) -> usize { self.0.push(l); self.0.len() }
}
fn walk<'d, W: Visit<'d>>(data: &'d [u8], w: &mut W) -> usize { w.list(data) }
static DATA: [u8; 2] = [1, 2];
fn main() {
    let v = vec![1u8, 2, 3];
    let mut c = Count(0);
    let mut k = Keep(Vec::new());
    println!("{} {}", walk(&v, &mut c), walk(&DATA, &mut k));
}
