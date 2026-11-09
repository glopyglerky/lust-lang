// expect: L0617
// rustc E0309: the impl header `Own<(T, &'a ())>` does not imply `T: 'a`.
trait A { type Out; }
struct Own<T>(T);
impl<'a, T> A for Own<(T, &'a ())> { type Out = &'a T; }
fn main() {}
