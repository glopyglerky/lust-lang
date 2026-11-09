// expect: L0610
// rustc E0283: two where clauses can prove the supertrait at
// different lifetimes, so the proof is ambiguous even though
// one of them matches exactly.
trait Tag<A> {}
trait NeedT<A>: Tag<A> {}
struct G<T>(T);
impl<'a, T> NeedT<&'a u8> for G<T> where G<T>: Tag<&'static u8>, G<T>: Tag<&'a u8> {}
fn main() {}
