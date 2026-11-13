// ARCH 2.6, accepted blanket Self forms. The bare-param form is
// the ts-rust AnyValue shape (frontend/json_ext.rs:43). The &T
// form is the MarshalerTo shape (frontend/json.rs:1660), which
// sits in the Ref bucket and is disjoint from Adt impls.
trait Base {
    fn v(&self) -> u8;
}
trait Blanket {
    fn b(&self) -> u8;
}
impl<T: Base> Blanket for T {
    fn b(&self) -> u8 {
        self.v() + 1
    }
}
trait Wrap {
    fn w(&self) -> u8;
}
struct S;
impl Base for S {
    fn v(&self) -> u8 {
        10
    }
}
impl Wrap for S {
    fn w(&self) -> u8 {
        20
    }
}
impl<T: Wrap + ?Sized> Wrap for &T {
    fn w(&self) -> u8 {
        (**self).w() + 1
    }
}
fn via_bound<X: Wrap>(v: &X) -> u8 {
    v.w()
}
fn main() {
    let s = S;
    let r = &s;
    // r.w() by method syntax autoderefs to S first (2.10), so the
    // &T blanket is exercised through a bound instead.
    println!("{} {} {}", s.b(), s.w(), via_bound(&r));
}
