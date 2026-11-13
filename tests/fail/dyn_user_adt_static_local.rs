// expect: L0307
// rustc E0515: `w`'s object lifetime is inferred, so the return type's `'static` reaches it,
// and the value borrows a local.
trait P { fn p(&self) -> u32; }
struct B<'b>(&'b u32);
impl<'b> P for B<'b> { fn p(&self) -> u32 { *self.0 } }
struct Own<X>(X);
fn mk() -> Own<Box<dyn P>> {
    let n = 3;
    let w: Own<Box<dyn P>> = Own(Box::new(B(&n)));
    w
}
fn main() { println!("{}", mk().0.p()); }
