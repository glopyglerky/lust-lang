// expect: L0617
// rustc E0308: the impl owes `S: Tag<&'a u8>` for every 'a. A
// where clause that proves the 'static case only does not
// prove it.
trait Tag<A> {}
trait Need<A>: Tag<A> {}
struct S;
impl Tag<&'static u8> for S {}
impl<'a> Need<&'a u8> for S where S: Tag<&'static u8> {}
fn main() {}
