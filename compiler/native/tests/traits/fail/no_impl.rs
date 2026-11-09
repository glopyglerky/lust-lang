// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.4 step 5: no candidate for a trait ref with a known
// head is an error that names the trait and the type.
trait T {}
fn need<X: T>() {}
struct S;
fn main() {
    need::<S>();
}
