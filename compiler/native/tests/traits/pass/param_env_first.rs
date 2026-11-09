// ARCH 2.4 step 1: param-env evidence is used first inside a
// generic body. No impl for the abstract T exists in the body;
// mono re-solves with the concrete type.
trait Tr {
    fn v(&self) -> u8;
}
struct A;
struct B;
impl Tr for A {
    fn v(&self) -> u8 {
        1
    }
}
impl Tr for B {
    fn v(&self) -> u8 {
        2
    }
}
fn call<T: Tr>(v: &T) -> u8 {
    v.v()
}
// Two bounds on two params: each call must use its own evidence.
fn pair<X: Tr, Y: Tr>(x: &X, y: &Y) -> u8 {
    x.v() + y.v()
}
fn main() {
    println!("{} {} {}", call(&A), call(&B), pair(&A, &B));
}
