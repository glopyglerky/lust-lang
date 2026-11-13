// expect: L0612
// rustc E0277: the literal falls back to i32, and the impl's nested bound `i32: Show` has no impl.
trait Show {
    fn show(&self) -> u32;
}
impl Show for u8 {
    fn show(&self) -> u32 {
        1
    }
}
impl Show for u16 {
    fn show(&self) -> u32 {
        2
    }
}
struct Wrap<T>(T);
impl<T: Show> Show for Wrap<T> {
    fn show(&self) -> u32 {
        self.0.show()
    }
}
fn need<S: Show>(s: S) -> u32 {
    s.show()
}
fn main() {
    println!("{}", need(Wrap(5)));
}
