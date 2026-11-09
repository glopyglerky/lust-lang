// expect: L0306
// rustc E0597: a default method stores through the impl's `'d`.
trait Store<'d> {
    fn put(&mut self, x: &'d u8);
    fn put_twice(&mut self, x: &'d u8) { self.put(x); self.put(x); }
}
struct V<'v>(Vec<&'v u8>);
impl<'d> Store<'d> for V<'d> { fn put(&mut self, x: &'d u8) { self.0.push(x); } }
fn main() { let mut v = V(Vec::new()); { let a = 1u8; v.put_twice(&a); } println!("{}", v.0.len()); }
