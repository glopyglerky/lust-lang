// Const generic values and array lengths keep their declared types
// (rustc types each as an anonymous const): unit struct values with
// const args, bool/char/signed const params, and array lengths built
// from const fns, const items, blocks, casts, and assoc consts.
#[derive(Clone, Copy, Debug, PartialEq)]
struct S<const N: usize>;
impl<const N: usize> S<N> {
    fn n(&self) -> usize {
        N
    }
    fn make() -> Self {
        Self
    }
}
struct B<const F: bool>;
trait Say {
    fn say(&self) -> &'static str;
}
impl Say for B<true> {
    fn say(&self) -> &'static str {
        "yes"
    }
}
impl Say for B<false> {
    fn say(&self) -> &'static str {
        "no"
    }
}
struct C<const X: char>;
impl<const X: char> C<X> {
    fn get(&self) -> char {
        X
    }
}
struct I<const N: i64>;
impl<const N: i64> I<N> {
    fn n(&self) -> i64 {
        N
    }
}
struct D<const N: usize = 4>;
impl<const N: usize> D<N> {
    fn n(&self) -> usize {
        N
    }
}
fn six(s: S<6>) -> usize {
    s.n()
}
fn plus_one<const M: usize>() -> usize {
    S::<M>.n() + 1
}
const fn twice(x: usize) -> usize {
    x * 2
}
const L: usize = 2;
const SMALL: u8 = 3;
struct T;
impl T {
    const N: usize = 2;
}
struct Arrs {
    a: [u8; twice(2)],
    b: [u8; L * 2 + 1],
    c: [u8; SMALL as usize],
    d: [u8; { let x = 2u8; x as usize + 1 }],
    e: [u8; T::N],
    f: [u8; (u32::BITS - 30) as usize],
    g: [u8; 1 << 2u8],
}
type A<const N: usize> = [u8; N];
fn main() {
    let a: S<3> = S;
    let b = S::<4>;
    println!("{} {} {} {}", a.n(), b.n(), S::<5>.n(), S::<7>::make().n());
    println!("{} {} {:?}", six(S), plus_one::<9>(), a == S::<3>);
    println!("{} {}", B::<true>.say(), B::<false>.say());
    println!("{} {} {}", C::<'q'>.get(), I::<{ -5 }>.n(), I::<-9>.n());
    let d: D = D;
    println!("{}", d.n());
    let r = Arrs { a: [0; 4], b: [0; 5], c: [0; 3], d: [0; 3], e: [0; 2], f: [0; 2], g: [0; 4] };
    println!("{} {} {} {} {} {} {}", r.a.len(), r.b.len(), r.c.len(), r.d.len(), r.e.len(), r.f.len(), r.g.len());
    let x: A<2> = [7, 8];
    println!("{:?}", x);
}
