// A path through a written Self type picks and checks the impl: a
// turbofish on the type, a non-generic alias at fixed args, `Self`,
// and two concrete impls that both define `tag`.
struct S<T>(T);
impl<T: Copy> S<T> {
    fn make(t: T) -> Self {
        S(t)
    }
    fn get(&self) -> T {
        self.0
    }
}
struct W<T>(T);
impl W<u8> {
    fn tag(&self) -> &'static str {
        "u8"
    }
}
impl W<u16> {
    fn tag(&self) -> &'static str {
        "u16"
    }
    fn both(&self) -> String {
        format!("{} {}", Self::tag(self), W::<u8>::tag(&W(0)))
    }
}
type A = S<u8>;
type B = W<u16>;
fn main() {
    let a = A::make(5);
    let b = S::<i64>::make(-3);
    let f = A::get;
    println!("{} {} {}", a.get(), b.get(), f(&a));
    println!("{} {}", W(1u8).tag(), W(2u16).tag());
    println!("{} {} {}", W::<u16>::tag(&W(3)), W::<u8>::tag(&W(4)), B::tag(&W(5)));
    let g = W::<u16>::tag;
    println!("{} [{}]", g(&W(6)), W(7u16).both());
}
