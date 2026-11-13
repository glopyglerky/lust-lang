// Clone and Copy hold for tuples of every arity (the 12-element limit is
// only for core's Debug/PartialEq/Eq/PartialOrd/Ord/Hash/Default impls).
// The auto traits are structural at every arity too.
#[derive(Clone, Copy)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}

fn c<T: Clone>(x: &T) -> T {
    x.clone()
}
fn k<T: Copy>(x: T) -> (T, T) {
    (x, x)
}
fn send<T: Send>(x: T) -> T {
    x
}

fn main() {
    let t = (1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8);
    let a = t;
    let b = t.clone();
    println!("{}", a.0 + a.12 + b.6 + t.3);
    let t16 = (2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8);
    let (x, y) = k(t16);
    println!("{}", x.15 + y.0 + t16.8);
    let s = S { t };
    let s2 = s;
    let s3 = c(&s);
    println!("{}", s.t.12 + s2.t.0 + s3.t.5);
    let t = send(t);
    println!("{}", t.12);
}
