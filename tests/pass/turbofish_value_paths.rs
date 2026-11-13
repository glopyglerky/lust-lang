// A turbofish on a struct literal, a tuple constructor, or an enum
// variant path fixes the type's params (a literal then takes them).
struct S<T> {
    a: T,
}
struct P<T>(T);
enum E<T> {
    A(T),
    B,
}

fn main() {
    let s = S::<u8> { a: 255 };
    let p = P::<u8>(255);
    let e = E::<u8>::A(255);
    let f = E::A::<u8>(255);
    let g = E::<u16>::B;
    let over = |x: u8| x.checked_add(1).is_none();
    let a = match e {
        E::A(v) => over(v),
        E::B => false,
    };
    let b = match f {
        E::A(v) => over(v),
        E::B => false,
    };
    let c = matches!(g, E::B);
    println!("{} {} {} {} {}", over(s.a), over(p.0), a, b, c);
}
