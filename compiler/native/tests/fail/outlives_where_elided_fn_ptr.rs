// expect: L0617
// rustc E0309: an elided `fn(&T)` param implies nothing about `'a`.
trait Tr<X> {}
impl<X> Tr<X> for () {}
fn f<'a, T>(_z: &'a (), _y: fn(&T)) where (): Tr<&'a T> {}
fn main() { f::<u8>(&(), |_| {}); }
