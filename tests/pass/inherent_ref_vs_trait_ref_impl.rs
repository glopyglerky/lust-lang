struct S;
impl S {
    fn m(&self) -> u8 {
        1
    }
}
trait Tr {
    fn m(self) -> u8;
}
impl Tr for &S {
    fn m(self) -> u8 {
        2
    }
}
fn main() {
    let s = S;
    let r = &s;
    println!("{} {}", s.m(), r.m());
}
