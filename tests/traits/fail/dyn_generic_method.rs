// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: a method with its own generic type params has no
// single vtable entry, so the trait is not dyn compatible.
trait T {
    fn g<U>(&self, x: U);
}
fn f(_x: &dyn T) {}
fn main() {}
