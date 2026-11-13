use std::fmt;
use std::ops::{Add, Deref};

struct S {
    v: i32,
}
impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "S<{}>", self.v)
    }
}
#[derive(Debug)]
struct E1;
#[derive(Debug)]
struct E2(String);
impl From<E1> for E2 {
    fn from(_: E1) -> E2 {
        E2("uno".to_string())
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
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n as u64 * 10)
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
        V(self.0 * o.0)
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
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.bytes
    }
}
fn wlen(w: &W) -> usize {
    w.len()
}
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("dropped {}", self.0);
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
fn main() {
    println!("{}", show(&S { v: 3 }));
    println!("{:?} {:?}", f(true).map_err(|e| e.0), f(false).map_err(|e| e.0));
    println!("{}", sum_counter());
    println!("{:?}", addv());
    println!("{}", wlen(&W { inner: "abc".to_string(), bytes: vec![1, 2, 3, 4, 5] }));
    println!("{}", hold());
}
