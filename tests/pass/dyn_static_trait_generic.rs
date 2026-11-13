// A `'static` trait bound on a type param proves a `'static` dyn target:
// `X: Tr` implies `X: 'static` when `trait Tr: 'static`.
trait Tr: 'static {
    fn r(&self) -> u32;
}
struct N(u32);
impl Tr for N {
    fn r(&self) -> u32 {
        self.0
    }
}
fn f<X: Tr>(x: &X) -> &dyn Tr {
    x
}
fn g<X: Tr + 'static>(x: &X) -> &(dyn Tr + 'static) {
    x
}
trait Plain {
    fn p(&self) -> u32;
}
impl Plain for N {
    fn p(&self) -> u32 {
        self.0 + 1
    }
}
fn h<X: Plain + 'static>(x: &X) -> &(dyn Plain + 'static) {
    x
}
fn main() {
    let n = N(5);
    println!("{} {} {}", f(&n).r(), g(&n).r(), h(&n).p());
}
