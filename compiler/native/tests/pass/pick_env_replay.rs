// A call a template proves through a where clause resolves once, as
// in rustc: instances replay that clause with its trait inputs, even
// where the concrete Self type has other impls (`From<u16>` next to
// the bound's `From<u8>`) or the input is an unsuffixed literal.
use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

trait Add2<X> {
    fn add2(&self, x: X) -> u32;
}
struct S;
impl Add2<u8> for S {
    fn add2(&self, x: u8) -> u32 {
        x as u32 + 1000
    }
}
impl Add2<u16> for S {
    fn add2(&self, x: u16) -> u32 {
        x as u32 + 2000
    }
}
fn go<U: Add2<u8>>(u: U) -> u32 {
    u.add2(8)
}

struct W(u64);
impl From<u8> for W {
    fn from(x: u8) -> W {
        W(x as u64 + 100)
    }
}
impl From<u16> for W {
    fn from(x: u16) -> W {
        W(x as u64 + 200)
    }
}
fn conv<U: From<u8>>() -> U {
    From::from(8)
}
fn conv_path<U: From<u8>>() -> U {
    U::from(9)
}
fn conv_float<U: From<f32>>() -> U {
    From::from(1.5)
}
fn widen<T: Into<u64>>(t: T) -> u64 {
    let x = t.into();
    x + 1
}
fn from_into<U: From<u8>>() -> U {
    8u8.into()
}

#[derive(Clone, Copy)]
struct M(u32);
impl PartialEq<u8> for M {
    fn eq(&self, o: &u8) -> bool {
        self.0 == *o as u32
    }
}
impl PartialEq<u16> for M {
    fn eq(&self, o: &u16) -> bool {
        self.0 == *o as u32 + 1
    }
}
impl PartialOrd<u8> for M {
    fn partial_cmp(&self, o: &u8) -> Option<Ordering> {
        self.0.partial_cmp(&(*o as u32))
    }
}
impl PartialOrd<u16> for M {
    fn partial_cmp(&self, o: &u16) -> Option<Ordering> {
        self.0.partial_cmp(&(*o as u32 + 100))
    }
}
fn eq5<T: PartialEq<u8>>(t: T) -> bool {
    t == 5
}
fn lt50<T: PartialOrd<u8>>(t: T) -> bool {
    t < 50
}

#[derive(Clone, Copy, Debug)]
struct V(i64);
impl Add<i8> for V {
    type Output = V;
    fn add(self, o: i8) -> V {
        V(self.0 + o as i64)
    }
}
impl Add<i16> for V {
    type Output = V;
    fn add(self, o: i16) -> V {
        V(self.0 + o as i64 * 1000)
    }
}
impl AddAssign<i8> for V {
    fn add_assign(&mut self, o: i8) {
        self.0 += o as i64;
    }
}
impl AddAssign<i16> for V {
    fn add_assign(&mut self, o: i16) {
        self.0 += o as i64 * 1000;
    }
}
fn bump<T: Add<i8, Output = T>>(t: T) -> T {
    t + 5
}
fn bump_assign<T: AddAssign<i8>>(mut t: T) -> T {
    t += 5;
    t
}
fn twice<X: From<u8> + Add<Output = X>>() -> X {
    conv::<X>() + conv()
}

fn main() {
    println!("{}", go(S));
    let a: u16 = conv();
    let b: u32 = conv();
    let w: W = conv();
    let p: W = conv_path();
    let f: f64 = conv_float();
    println!("{} {} {} {} {}", a, b, w.0, p.0, f);
    let i: u16 = from_into();
    println!("{} {} {}", widen(5u8), widen(7u32), i);
    println!("{} {} {} {}", eq5(M(5)), eq5(M(6)), lt50(M(10)), lt50(M(60)));
    let t: u32 = twice();
    let d: f64 = twice();
    println!("{:?} {:?} {} {}", bump(V(1)), bump_assign(V(1)), t, d);
}
