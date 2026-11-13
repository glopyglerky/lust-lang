// expect: L0612
// rustc E0310: `trait Tr: 'static` needs `X: 'static` on the impl.
trait Tr: 'static {}
struct W<X>(X);
impl<X> Tr for W<X> {}
fn main() {}
