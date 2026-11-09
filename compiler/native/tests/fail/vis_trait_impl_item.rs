// expect: L0001
trait Tr {
    fn t(&self) -> i32;
}
struct S;
impl Tr for S {
    pub fn t(&self) -> i32 {
        1
    }
}
fn main() {
    println!("{}", S.t());
}
