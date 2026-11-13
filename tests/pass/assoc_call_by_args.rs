// `Type::f(args)` where only the args fix the trait's inputs: rustc
// types the call by the trait's signature and selects the impl once
// the args bind them (`From` with core's identity impl beside the
// user's, or two impls of `Conv<T>`).
#[derive(Debug)]
struct O {
    v: u8,
}
#[derive(Debug)]
struct S {
    n: usize,
}
impl From<O> for S {
    fn from(o: O) -> S {
        S { n: o.v as usize }
    }
}
impl From<&O> for S {
    fn from(o: &O) -> S {
        S { n: o.v as usize + 10 }
    }
}
impl<const N: usize> From<[u8; N]> for S {
    fn from(a: [u8; N]) -> S {
        S { n: N * 100 + a[0] as usize }
    }
}
impl<T: Copy> From<(T, T)> for S {
    fn from(_p: (T, T)) -> S {
        S { n: 2 }
    }
}
impl From<&str> for S {
    fn from(s: &str) -> S {
        S { n: s.len() }
    }
}
struct W<T> {
    v: Vec<T>,
}
impl<T> From<Vec<T>> for W<T> {
    fn from(v: Vec<T>) -> W<T> {
        W { v }
    }
}
trait Conv<T> {
    fn conv(t: T) -> Self;
    fn twice(t: T) -> (Self, Self)
    where
        T: Copy,
        Self: Sized,
    {
        (Self::conv(t), Self::conv(t))
    }
}
impl Conv<u8> for S {
    fn conv(t: u8) -> S {
        S { n: t as usize }
    }
}
impl Conv<u16> for S {
    fn conv(t: u16) -> S {
        S { n: t as usize + 1000 }
    }
}
trait Mk<T> {
    fn mk<U: Copy + Into<u32>>(t: T, u: U) -> Self;
}
impl Mk<u8> for S {
    fn mk<U: Copy + Into<u32>>(t: u8, u: U) -> S {
        S { n: t as usize + u.into() as usize }
    }
}
impl Mk<O> for S {
    fn mk<U: Copy + Into<u32>>(t: O, u: U) -> S {
        S { n: t.v as usize * 10 + u.into() as usize }
    }
}
trait Out<T> {
    type Res;
    fn out(t: T) -> Self::Res;
}
impl Out<u8> for S {
    type Res = u16;
    fn out(t: u8) -> u16 {
        t as u16 * 2
    }
}
impl Out<u16> for S {
    type Res = u32;
    fn out(t: u16) -> u32 {
        t as u32 * 3
    }
}
fn via<T>(t: T) -> S
where
    S: Conv<T>,
{
    S::conv(t)
}
fn both<T: Conv<u8> + Conv<u16>>() -> (T, T) {
    (T::conv(1u8), T::conv(2u16))
}
fn main() {
    let o = O { v: 4 };
    println!("{:?} {:?} {:?}", S::from(&o), S::from(o), S::from([7u8, 8, 9]));
    println!("{:?} {:?}", S::from((1u8, 2u8)), S::from("hello"));
    println!("{}", W::from(vec![1, 2, 3]).v.len());
    println!("{:?} {:?} {:?}", S::conv(3u8), S::conv(4u16), <S as Conv<u16>>::twice(5));
    println!("{:?} {:?}", S::mk(1u8, 5u16), S::mk(O { v: 2 }, 7u8));
    println!("{} {}", S::out(5u8), S::out(5u16));
    println!("{:?} {:?}", via(7u8), via(8u16));
    let (a, b): (S, S) = both();
    println!("{:?} {:?}", a, b);
}
