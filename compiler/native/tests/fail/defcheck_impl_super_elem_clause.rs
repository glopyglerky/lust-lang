// expect: L0617
// rustc E0477: the impl owes `S: Tk<(&'a u32, &'a u32)>`. The
// where clause names Tk at the element type, which proves
// nothing about the pair; the only Tk impl needs 'static.
trait Tk<A> {}
trait Sup<A>: Tk<(A, A)> {}
struct S;
impl<T: 'static> Tk<T> for S {}
impl<'a> Sup<&'a u32> for S where S: Tk<&'a u32> {}
fn main() {}
