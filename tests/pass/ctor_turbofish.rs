// Generic arguments written on constructor paths fix the Adt's
// params: tuple structs, struct literals, enum variants with the
// args on the enum or the variant segment, imported variants, and
// unit variants of generic enums.
mod m {
    pub struct W<A>(pub A);
}
struct T<A>(A, usize);
struct S<A, B> {
    a: A,
    b: B,
}
enum E<A> {
    One(A),
    Two { a: A, b: A },
    Zero,
}
use E::*;
struct R<'a, A>(&'a A);
fn main() {
    let t = T::<u8>(250, 2);
    let s = S::<u8, _> { a: 255, b: "x" };
    let w = m::W::<u8>(250);
    println!("{} {} {} {}", t.0.wrapping_add(10), s.a.wrapping_add(1), s.b, w.0.wrapping_add(9));
    let v = E::One::<u8>(250);
    if let E::One(x) = v {
        println!("{}", x.wrapping_add(9));
    }
    let i = One::<u8>(250);
    let y = match i {
        One(x) => x,
        _ => 0,
    };
    println!("{}", y.wrapping_add(9));
    let f = E::<i8>::Two { a: -1, b: 127 };
    if let E::Two { a, b } = f {
        println!("{}", a.wrapping_add(b));
    }
    let z: E<u8> = E::Zero;
    let z2 = E::<u16>::Zero;
    let z3: E<char> = Zero;
    let n = match (z, z2, z3) {
        (E::Zero, E::Zero, E::Zero) => 3,
        _ => 0,
    };
    let x = 5u8;
    let r = R::<'_, u8>(&x);
    println!("{} {}", n, r.0);
}
