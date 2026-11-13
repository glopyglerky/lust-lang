// `Trait::f(args)` for a fn with no `self`: Self comes from the
// expected type, or a later use fixes it (rustc infers it the same way).
trait Mk {
    fn mk(n: u8) -> Self;
    fn mk2() -> Self
    where
        Self: Sized,
    {
        Self::mk(2)
    }
}
trait Conv<T> {
    fn conv(t: T) -> Self;
}
trait Make {
    fn make<U: Into<u32>>(u: U) -> Self;
}
#[derive(Debug)]
struct S {
    n: u8,
}
#[derive(Debug)]
struct T {
    n: u16,
}
struct W {
    s: S,
}
impl Drop for T {
    fn drop(&mut self) {
        println!("drop T {}", self.n);
    }
}
impl Mk for S {
    fn mk(n: u8) -> S {
        S { n }
    }
}
impl Mk for T {
    fn mk(n: u8) -> T {
        T { n: n as u16 + 300 }
    }
}
impl Conv<u8> for S {
    fn conv(t: u8) -> S {
        S { n: t }
    }
}
impl Conv<u16> for S {
    fn conv(t: u16) -> S {
        S { n: t as u8 + 100 }
    }
}
impl Make for S {
    fn make<U: Into<u32>>(u: U) -> S {
        S { n: u.into() as u8 }
    }
}
fn show(s: S) -> u8 {
    s.n
}
fn take(_s: &S, _t: &T) {}
fn f<X: Mk>() -> X {
    Mk::mk(1)
}
fn g<X: Mk>() -> X {
    let x = Mk::mk(2);
    x
}
fn main() {
    let a: S = Mk::mk(8);
    let b: T = Mk::mk(2);
    println!("{} {} {}", a.n, b.n, show(Mk::mk(5)));
    let mut v = Vec::new();
    v.push(Mk::mk(3));
    let first: &S = &v[0];
    println!("{:?}", first);
    let c: S = Conv::conv(3u16);
    let d: S = Make::make(4u16);
    let e: S = Mk::mk2();
    println!("{} {} {}", c.n, d.n, e.n);
    let x = Mk::mk(6);
    let w = W { s: x };
    println!("{}", w.s.n);
    let l: Vec<S> = vec![Mk::mk(1), Mk::mk(2)];
    println!("{:?}", l);
    let p = Mk::mk(1);
    let q = Mk::mk(2);
    take(&p, &q);
    println!("{:?} {:?}", p, q);
    let s1: S = f();
    let s2: S = g();
    println!("{} {}", s1.n, s2.n);
}
