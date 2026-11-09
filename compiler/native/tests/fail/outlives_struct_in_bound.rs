// expect: L0617
// rustc E0309: `W<'a, T>` needs what its field `&'a T` needs.
struct W<'a, T> { r: &'a T }
trait Tr<X> {}
impl<X> Tr<X> for () {}
fn f<'a, T, U: Tr<W<'a, T>>>(_z: &'a ()) {}
fn main() { f::<u8, ()>(&()); }
