// A wrapper impl with a lifetime-trait where-clause over a `'static` impl, fed `'static` data.
trait Tr<'a> { fn put(&mut self, x: &'a u8); }
struct Stat(Vec<&'static u8>);
impl Tr<'static> for Stat { fn put(&mut self, x: &'static u8) { self.0.push(x); } }
struct Wrap<T>(T);
impl<'a, T: Tr<'a>> Tr<'a> for Wrap<T> { fn put(&mut self, x: &'a u8) { self.0.put(x); } }
fn feed<'d, V: Tr<'d>>(w: &mut V, x: &'d u8) { w.put(x); }
static L: u8 = 5;
fn main() { let mut w = Wrap(Stat(Vec::new())); feed(&mut w, &L); println!("{}", (w.0).0.len()); }
