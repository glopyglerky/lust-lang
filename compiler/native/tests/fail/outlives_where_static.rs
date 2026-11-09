// expect: L0617
// rustc E0310: `&'static T` in a bound needs `T: 'static`.
trait Tr<X> {}
impl<X> Tr<X> for () {}
fn f<T>() where (): Tr<&'static T> {}
fn main() { f::<u8>(); }
