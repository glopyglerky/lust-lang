// Written-Self paths on one warm daemon: the alias's target args
// change (the call's arg type follows), a bad arg (an error), a
// generic alias, and one of two same-named impls removed (an error).
struct S<T>(T);
impl<T: Copy> S<T> {
    fn make(t: T) -> Self {
        S(t)
    }
}
struct W<T>(T);
impl W<u8> {
    fn tag() -> &'static str {
        "u8"
    }
}
impl W<u16> {
    fn tag() -> &'static str {
        "u16"
    }
}
type A = S<u16>;
fn main() {
    let a = A::make(300);
    println!("{} {}", a.0, W::<u16>::tag());
}
