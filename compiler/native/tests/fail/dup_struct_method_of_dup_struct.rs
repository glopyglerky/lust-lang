// expect: L0106
struct S {
    x: i64,
}
struct S {
    y: i64,
}
impl S {
    fn a(&self) {}
}
fn main() {}
