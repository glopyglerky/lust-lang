trait Mark {}
struct L;
struct W<T>(T);
impl<T: Mark> W<T> {
    fn f(&self) -> u8 {
        1
    }
}
impl W<L> {
    fn f(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", W(L).f());
}
