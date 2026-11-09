// A deref coercion through user `Deref` impls whose last target is a
// `Vec<T>` or `String`, then that type's deref to `[T]` / `str` (ts-rust
// `&LAZY_TABLE` passed where `&[T]` is expected).
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;
static T: LazyLock<Vec<u8>> = LazyLock::new(|| vec![1, 2, 3]);
static S: LazyLock<String> = LazyLock::new(|| String::from("hi"));
struct W<X>(X);
impl<X> Deref for W<X> {
    type Target = X;
    fn deref(&self) -> &X {
        &self.0
    }
}
impl<X> DerefMut for W<X> {
    fn deref_mut(&mut self) -> &mut X {
        &mut self.0
    }
}
fn f(x: &[u8]) -> usize {
    x.len()
}
fn g(s: &str) -> usize {
    s.len()
}
fn h(v: &Vec<u8>) -> usize {
    v.len()
}
fn m(x: &mut [u8]) {
    x[0] = 42;
}
fn main() {
    println!("{} {} {}", f(&T), g(&S), h(&T));
    let r: &[u8] = &T;
    println!("{:?}", r);
    let mut w = W(W(vec![5u8, 6]));
    m(&mut w);
    println!("{} {:?}", f(&w), w.0.0);
    let ws = W(String::from("abc"));
    let s: &str = &ws;
    println!("{} {}", s, g(&&ws));
}
