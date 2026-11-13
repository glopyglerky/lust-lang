// expect: L0106
trait T {
    type A;
}
struct S;
impl T for S {
    type A = i64;
    type A = i32;
}
fn main() {}
