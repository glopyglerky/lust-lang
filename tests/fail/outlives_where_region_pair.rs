// expect: L0617
// rustc E0491: `&'a &'b ()` needs `'b: 'a`.
trait Tr<X> {}
impl<X> Tr<X> for () {}
fn f<'a, 'b>(_x: &'a (), _y: &'b ()) where (): Tr<&'a &'b ()> {}
fn main() { f(&(), &()); }
