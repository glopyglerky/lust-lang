// expect: L0306
// rustc E0597: through `impl<'a, T: Tr<'a>> Tr<'a> for Wrap<T>`, `Wrap<Stat>` takes `'static` data (only `impl Tr<'static> for Stat`).
trait Tr<'a> { fn put(&mut self, x: &'a u8); }
struct Stat(Vec<&'static u8>);
impl Tr<'static> for Stat { fn put(&mut self, x: &'static u8) { self.0.push(x); } }
struct Wrap<T>(T);
impl<'a, T: Tr<'a>> Tr<'a> for Wrap<T> { fn put(&mut self, x: &'a u8) { self.0.put(x); } }
fn feed<'d, V: Tr<'d>>(w: &mut V, x: &'d u8) { w.put(x); }
fn main() { let mut w = Wrap(Stat(Vec::new())); let l = 5u8; feed(&mut w, &l); println!("{}", (w.0).0.len()); }
