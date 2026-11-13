// expect: L0612
// rustc E0277: `B: Tr` does not hold for a bounded const fn in a static.
use std::marker::PhantomData;
trait Tr {}
struct A;
struct B;
impl Tr for A {}
struct K<T>(u32, PhantomData<T>);
impl<T: Tr> K<T> {
    const fn new(v: u32) -> K<T> {
        K(v, PhantomData)
    }
}
static OK: K<A> = K::new(1);
static BAD: K<B> = K::new(2);
fn main() {
    println!("{} {}", OK.0, BAD.0);
}
