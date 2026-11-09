// expect: L0203
// rustc E0496: a `for<'a>` binder may not shadow a lifetime in scope.
struct P<'a>(&'a u8);
impl<'a> P<'a> {
    fn m(&self, f: for<'a> fn(&'a u8) -> u8) -> u8 { f(self.0) }
}
fn main() { println!("{}", P(&3).m(|x| *x)); }
