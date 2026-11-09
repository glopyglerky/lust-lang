// `impl<'d> Tr<'d> for Keep<'d>`: a generic call's `'d` is the region of `Keep` it binds.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]); fn n(&self) -> usize; }
struct Keep<'k>(Vec<&'k [u8]>);
impl<'d> Visit<'d> for Keep<'d> { fn list(&mut self, l: &'d [u8]) { self.0.push(l); } fn n(&self) -> usize { self.0.len() } }
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) { w.list(data); }
fn main() {
    let a = vec![1u8];
    let mut k = Keep(Vec::new());
    feed(&mut k, &a);
    let b = vec![2u8, 3];
    feed(&mut k, &b);
    println!("{}", k.n());
}
