// expect: L0101
// rustc E0425: the same blowup as error_type_obligation_blowup.rs, with a local trait whose
// `&X` impl the error type matched.
trait T {}
impl T for i32 {}
impl T for u8 {}
impl<X: T + ?Sized> T for &X {}
fn g<P: T>(_p: P) {}
fn main() {
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
    g(&target);
}
