// expect: L0702
// Valid Rust: a dyn coercion in a static initializer needs a vtable at
// compile time, which lust's const evaluator does not build.
trait T {
    fn read(&self) -> u32;
}
struct S {
    n: u32,
}
impl T for S {
    fn read(&self) -> u32 {
        self.n
    }
}
static S0: S = S { n: 4 };
static D: &(dyn T + Sync) = &S0;
fn main() {
    println!("{}", D.read());
}
