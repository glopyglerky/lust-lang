// expect: L0617
// rustc E0309: a trait's where clause assumes nothing.
trait Tr<X> {}
impl<X> Tr<X> for () {}
trait U<'a, T> where (): Tr<&'a T> {}
fn main() {}
