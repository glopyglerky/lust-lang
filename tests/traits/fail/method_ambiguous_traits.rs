// rustc-error: E0034
// lust-expect: L0720
// ARCH 2.10 step 2: two trait hits in the same probe with
// different traits are ambiguous. The fix is a fully qualified
// call, which the lust hint must name.
trait A {
    fn m(&self) -> u8;
}
trait B {
    fn m(&self) -> u8;
}
struct S;
impl A for S {
    fn m(&self) -> u8 {
        1
    }
}
impl B for S {
    fn m(&self) -> u8 {
        2
    }
}
fn main() {
    println!("{}", S.m());
}
