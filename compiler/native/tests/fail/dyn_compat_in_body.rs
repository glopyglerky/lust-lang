// expect: L0701
// ARCH 2.8, rustc E0038: the check runs at a dyn type written in a body (a cast).
trait Bad { fn copy(&self) -> Self; }
struct S;
impl Bad for S { fn copy(&self) -> Self { S } }
fn main() { let s = S; let d = &s as &dyn Bad; }
