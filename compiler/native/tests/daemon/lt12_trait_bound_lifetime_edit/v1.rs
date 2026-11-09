// A region-only edit of a bound's lifetime argument (`V: Visit<'d>` to
// `V: Visit<'static>`): the unchanged `main` must recheck (W-BORROW,
// `RegionSig::trait_bounds` in the region-sig fingerprint). With only
// `impl Visit<'static> for K`, the `'d` bound makes `main`'s borrow
// `'static`; the `'static` bound does not.
trait Visit<'d> {
    fn n(&self) -> usize;
}
struct K(usize);
impl Visit<'static> for K {
    fn n(&self) -> usize {
        self.0
    }
}
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) -> usize {
    w.n() + data.len()
}
fn main() {
    let mut k = K(1);
    let v = vec![1u8, 2];
    println!("{}", feed(&mut k, &v));
}
