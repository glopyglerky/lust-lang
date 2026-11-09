// expect: L0106
trait T {
    fn a(&self);
}
struct S;
impl T for S {
    fn a(&self) {}
    fn a(&self) {}
}
fn main() {}
