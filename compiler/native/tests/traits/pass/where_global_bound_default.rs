// A global where-bound (`A: Tr`, no type parameter) on a plain fn: the
// impl answers it, so `go` is A's method and `dflt` the trait default.
trait Tr {
    fn go(&self) -> i32;
    fn dflt(&self) -> i32 {
        100
    }
}
struct A;
impl Tr for A {
    fn go(&self) -> i32 {
        1
    }
}
fn needs() -> i32
where
    A: Tr,
{
    A.dflt() + A.go()
}
fn main() {
    println!("{}", needs());
}
