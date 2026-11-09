// A region-only edit of an impl header (`impl Visit<'static>` to
// `impl<'d> Visit<'d>`): the unchanged `main` must recheck (W-BORROW,
// `RegionTables::impl_heads` in the impl's content fingerprint).
trait Visit<'d> {
    fn list(&mut self, l: &'d [u8]) -> usize;
}
struct K(usize);
impl<'d> Visit<'d> for K {
    fn list(&mut self, l: &'d [u8]) -> usize {
        self.0 += l.len();
        self.0
    }
}
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) -> usize {
    w.list(data)
}
fn main() {
    let mut k = K(0);
    let v = vec![1u8, 2];
    println!("{}", feed(&mut k, &v));
}
