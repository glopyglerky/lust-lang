// A user From impl reached through an `Into<U>` bound in a generic
// instance: U comes from the bound, also where nothing else expects a
// type (`t.into().0`).
struct A(i32);
impl From<i32> for A {
    fn from(x: i32) -> A {
        A(x)
    }
}
struct S(String);
impl From<&str> for S {
    fn from(x: &str) -> S {
        S(x.to_string())
    }
}
fn take<T: Into<A>>(t: T) -> i32 {
    t.into().0
}
fn takes(t: impl Into<S>) -> String {
    t.into().0
}
fn main() {
    let a: A = 3.into();
    let s: S = "x".into();
    println!("{} {} {} {}", a.0, s.0, take(5), takes("y"));
    println!("{}", take(A(6)));
}
