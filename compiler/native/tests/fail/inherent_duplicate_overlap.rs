// expect: L0106
struct W<T>(T);
impl<T> W<T> {
    fn m(&self) -> u8 {
        1
    }
}
impl W<u8> {
    fn m(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", W(1u8).m());
}
