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
type A<T> = S<T>;
fn main() {
    let a = A::<u16>::make(300u16);
    println!("{} {}", a.0, W::<u16>::tag());
}
