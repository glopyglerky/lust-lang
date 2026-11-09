// expect: L0306
// rustc E0597: a forwarded bound's `'x` outlives a local.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]); }
fn feed<'d, V: Visit<'d>>(w: &mut V, data: &'d [u8]) { w.list(data); }
fn outer<'x, W: Visit<'x>>(w: &mut W, d: &'x [u8]) { feed(w, d); }
fn bad<'x, W: Visit<'x>>(w: &mut W) { let local = vec![1u8]; feed(w, &local); }
fn main() {}
