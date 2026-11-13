// expect: L0106
trait T {
    fn a(&self);
}
trait T {
    fn b(&self);
}
struct S;
impl T for S {
    fn a(&self) {}
}
fn main() {}
