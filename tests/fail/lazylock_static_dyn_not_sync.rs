// expect: L0612
// A `dyn Tr` without `Sync` makes `LazyLock<Box<dyn Tr + Send>>` not `Sync`.
use std::sync::LazyLock;
trait Tr {
    fn v(&self) -> u32;
}
struct A;
impl Tr for A {
    fn v(&self) -> u32 {
        1
    }
}
static L: LazyLock<Box<dyn Tr + Send>> = LazyLock::new(|| Box::new(A));
fn main() {
    println!("{}", L.v());
}
