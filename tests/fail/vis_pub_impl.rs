// expect: L0001
struct S;
pub impl S {
    fn t(&self) -> i32 {
        1
    }
}
fn main() {
    println!("{}", S.t());
}
