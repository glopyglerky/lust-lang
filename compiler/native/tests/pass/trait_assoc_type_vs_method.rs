trait T {
    type A;
    fn A(&self) -> Self::A;
}
struct S;
impl T for S {
    type A = i64;
    fn A(&self) -> i64 {
        6
    }
}
fn main() {
    let s = S;
    println!("{}", s.A());
}
