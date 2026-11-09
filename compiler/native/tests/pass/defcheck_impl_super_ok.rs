// Supertrait obligations the impl does prove: a blanket Tag
// impl at the same lifetime, and a where clause at the exact
// arguments.
trait Tag<A> {}
trait Need<A>: Tag<A> {}
struct S;
impl<'a> Tag<&'a u8> for S {}
impl<'a> Need<&'a u8> for S {}
struct W<X>(X);
impl<'a, X> Need<&'a X> for W<X> where W<X>: Tag<&'a X> {}
impl<'a, X> Tag<&'a X> for W<X> {}
fn want<T: Need<&'static u8>>(_t: &T) -> u8 { 3 }
fn main() {
    println!("{}", want(&S) + want(&W(0u8)));
}
