use std::fmt;
use std::ops::{Add, Deref};

struct S {
    v: i32,
}
impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "S[{}]", self.v)
    }
}
#[derive(Debug)]
struct E1;
#[derive(Debug)]
struct E2(String);
impl From<E1> for E2 {
    fn from(_: E1) -> E2 {
        E2("eins".to_string())
    }
}
fn g(ok: bool) -> Result<i32, E1> {
    if ok { Ok(1) } else { Err(E1) }
}
fn f(ok: bool) -> Result<i32, E2> {
    let x = g(ok)?;
    Ok(x + 1)
}
struct Counter {
    n: u32,
}
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n * 7)
        } else {
            None
        }
    }
}
fn sum_counter() -> u32 {
    let mut t = 0u32;
    for x in (Counter { n: 0 }) {
        t += x as u32;
    }
    t
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct V(i32);
impl Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        V(self.0 + o.0)
    }
}
fn addv() -> V {
    V(1) + V(2)
}
struct W {
    inner: String,
    bytes: Vec<u8>,
}
impl Deref for W {
    type Target = String;
    fn deref(&self) -> &String {
        &self.inner
    }
}
fn wlen(w: &W) -> usize {
    w.len()
}
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct Holder {
    d: D,
    n: i32,
}
fn hold() -> i32 {
    let h = Holder { d: D("h"), n: 4 };
    h.n
}
fn show(s: &S) -> String {
    format!("{}", s)
}
fn show_all<T: fmt::Display>(xs: &[T]) -> String {
    xs.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(",")
}
fn total<I: Iterator<Item = u32>>(it: I) -> u32 {
    it.sum()
}
fn dbl<T: Add<Output = T> + Copy>(x: T) -> T {
    x + x
}
fn conv<A, B: From<A>>(a: A) -> B {
    B::from(a)
}
fn keep<T>(x: T) -> T {
    x
}
fn main() {
    println!("{}", show_all(&[S { v: 1 }, S { v: 2 }]));
    let e: E2 = conv(E1);
    println!("{}", e.0);
    println!("{}", total(Counter { n: 0 }));
    println!("{:?}", dbl(V(5)));
    println!("{}", wlen(&W { inner: "abcd".to_string(), bytes: vec![1] }));
    let k = keep(Holder { d: D("k"), n: 7 });
    println!("{}", k.n);
}
