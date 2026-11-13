// rustc-error: E0038
// lust-expect: L0701
// ARCH 2.8: a trait that requires Self: Sized is not dyn
// compatible, because a dyn value is unsized.
trait T: Sized {
    fn v(&self) -> u8;
}
fn f(_x: &dyn T) {}
fn main() {}
