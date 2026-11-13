// expect: L0106
struct S;
impl S {
    fn m(&self) -> u8 {
        1
    }
}
impl S {
    fn m(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", S.m());
}
