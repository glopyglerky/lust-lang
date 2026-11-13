// rustc-error: E0275
// lust-expect: L0611
// ARCH 2.4 step 7: re-entering the same trait ref on the stack
// is an overflow, not an infinite loop.
trait T {}
impl<X> T for Vec<X> where Vec<X>: T {}
fn need<X: T>() {}
fn main() {
    need::<Vec<u8>>();
}
