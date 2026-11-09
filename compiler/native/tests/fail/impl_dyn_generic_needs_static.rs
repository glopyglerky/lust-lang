// expect: L0733
// rustc E0521: `impl dyn Tr` is for `dyn Tr + 'static`, and `a` has a shorter object lifetime.
trait Tr {
    fn id(&self) -> u8;
}
impl Tr for u8 {
    fn id(&self) -> u8 {
        *self
    }
}
impl dyn Tr {
    fn twice<T: From<u8>>(&self) -> T {
        T::from(self.id() * 2)
    }
}
fn go<'a>(a: &'a (dyn Tr + 'a)) -> u16 {
    a.twice::<u16>()
}
fn main() {
    println!("{}", go(&1u8));
}
