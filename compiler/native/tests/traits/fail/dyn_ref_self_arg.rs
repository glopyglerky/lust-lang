// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: Self in a non-receiver argument fails even behind a
// reference. There is no allowance for &Self there.
// From the sol probe r2-dyn-self-ref.rs.
trait Bad {
    fn other(&self, value: &Self);
}
fn use_object(_x: &dyn Bad) {}
fn main() {}
