// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: Self as a return type is not dispatchable.
// From the sol probe dyn_self.rs.
trait Bad {
    fn copy(&self) -> Self;
}
fn f(_x: &dyn Bad) {}
fn main() {}
