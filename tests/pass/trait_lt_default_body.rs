// A default body calls the trait's own methods through `Self` with the trait's lifetime.
trait Store<'d> {
    fn put(&mut self, x: &'d u8);
    fn put_twice(&mut self, x: &'d u8) { self.put(x); self.put(x); }
}
struct V<'v>(Vec<&'v u8>);
impl<'d> Store<'d> for V<'d> { fn put(&mut self, x: &'d u8) { self.0.push(x); } }
fn main() { let a = 1u8; let mut v = V(Vec::new()); v.put_twice(&a); println!("{}", v.0.len()); }
