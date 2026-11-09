// expect: L0720
trait A {
    const C: i32 = 1;
}
trait B {
    const C: i32 = 2;
}
struct S;
impl A for S {}
impl B for S {}
fn g<T>(t: T) -> i32 {
    let _ = &t;
    S::C
}
fn main() {
    println!("{}", g(3));
}
