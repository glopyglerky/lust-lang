// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: a trait with an associated const is not dyn
// compatible. The vtable has no slot for a const value.
trait T {
    const C: u8;
}
fn f(_x: &dyn T) {}
fn main() {}
