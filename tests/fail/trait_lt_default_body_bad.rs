// expect: L0306
// rustc E0597: a default body passes a local where the trait takes `&'d`.
trait Store<'d> {
    fn put(&mut self, x: &'d u8);
    fn bad(&mut self) { let z = 1u8; self.put(&z); }
}
fn main() {}
