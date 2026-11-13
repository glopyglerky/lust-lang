// ARCH 2.10 step 2: receiver forms are probed in order (by value,
// then &S, then &mut S) at each autoderef step. A trait by-value
// method therefore beats an inherent &self method: the by-value
// form is probed first. Verified with rustc 1.99 before pinning.
struct S;
impl S {
    fn m(&self) -> u8 {
        1
    }
}
trait T {
    fn m(self) -> u8;
}
impl T for S {
    fn m(self) -> u8 {
        2
    }
}
fn main() {
    let s = S;
    println!("{}", s.m());
}
