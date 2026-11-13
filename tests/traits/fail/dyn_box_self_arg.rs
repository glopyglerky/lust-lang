// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: Self itself, even boxed, is E0038 in an argument.
// Only the erased object type (Box<dyn T>) is allowed there.
// From the round-2 probe dyn3.rs.
trait T {
    fn f(&self, x: Box<Self>);
}
fn u(_x: &dyn T) {}
fn main() {}
