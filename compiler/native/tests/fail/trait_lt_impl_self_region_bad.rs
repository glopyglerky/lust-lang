// expect: L0306
// rustc E0597: `impl<'d> Tr<'d> for Keep<'d>` stores the data for `Keep`'s region.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]); fn n(&self) -> usize; }
struct Keep<'k>(Vec<&'k [u8]>);
impl<'d> Visit<'d> for Keep<'d> { fn list(&mut self, l: &'d [u8]) { self.0.push(l); } fn n(&self) -> usize { self.0.len() } }
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) { w.list(data); }
fn main() {
    let mut k = Keep(Vec::new());
    { let b = vec![2u8, 3]; feed(&mut k, &b); }
    println!("{}", k.n());
}
