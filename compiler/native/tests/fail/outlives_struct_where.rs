// expect: L0617
// rustc E0309: the fields `&'a ()` and `T` do not give `T: 'a`.
trait Tr<X> {}
impl<X> Tr<X> for () {}
struct S<'a, T>(&'a (), T) where (): Tr<&'a T>;
fn main() {}
