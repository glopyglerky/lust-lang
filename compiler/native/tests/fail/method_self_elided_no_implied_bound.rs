// expect: L0617
// rustc E0309: `&self` names no `'a`, so nothing says `T: 'a` for `impl Fn(&'a T)`.
struct C<T> { v: Vec<T> }
impl<T> C<T> {
    fn e<'a>(&self, pick: impl Fn(&'a T)) {}
}
fn main() {}
