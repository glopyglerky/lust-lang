// expect: L0203
// rustc E0309: the caller's `U` has no `U: 'b` bound, which `wrap` needs.
trait Show {
    fn show(&self) -> String;
}
impl<T: std::fmt::Debug> Show for T {
    fn show(&self) -> String {
        format!("{:?}", self)
    }
}
fn wrap<'a, T: Show + 'a>(t: T) -> Box<dyn Show + 'a> {
    Box::new(t)
}
fn g<'b, U: Show>(u: U) -> Box<dyn Show + 'b> {
    wrap(u)
}
fn main() {
    println!("{}", g(5u8).show());
}
