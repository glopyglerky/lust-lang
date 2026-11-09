// expect: L0001
trait Tr {
    pub fn t(&self) -> i32;
}
struct S;
impl Tr for S {
    fn t(&self) -> i32 {
        1
    }
}
fn main() {
    println!("{}", S.t());
}
