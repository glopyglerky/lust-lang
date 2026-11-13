// expect: L0617
// rustc E0309: a where clause type `&'a T` with only `&'a ()` in the signature.
trait Tr<X> {}
impl<X> Tr<X> for () {}
fn f<'a, T>(_z: &'a ()) where (): Tr<&'a T> {}
fn main() { f::<u8>(&()); }
