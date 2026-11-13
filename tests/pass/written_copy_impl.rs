// A written `impl Copy` (no derive) makes the type Copy where its header
// matches and its bounds hold (ts-rust ast/source_file_ls.rs:49: an
// `impl<T> Copy` over a PhantomData key has no `T: Copy` bound).
use std::marker::PhantomData;
pub struct Key<T> {
    key: u64,
    _t: PhantomData<fn() -> T>,
}
impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Key<T> {}
struct NoClone;
struct P(u8, char);
impl Clone for P {
    fn clone(&self) -> P {
        *self
    }
}
impl Copy for P {}
struct G<T>(T, u8);
impl<T: Clone> Clone for G<T> {
    fn clone(&self) -> Self {
        G(self.0.clone(), self.1)
    }
}
impl<T> Copy for G<T> where T: Copy {}
struct R<'a> {
    s: &'a str,
}
impl<'a> Clone for R<'a> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a> Copy for R<'a> {}
#[derive(Clone)]
struct Q(u8);
impl Copy for Q {}
#[derive(Clone, Copy)]
struct W(Q);
fn twice<T>(k: &G<T>) -> (G<T>, G<T>) where G<T>: Copy {
    (*k, *k)
}
fn key_of<T>(k: &Key<T>) -> Key<T> {
    *k
}
fn main() {
    let k: Key<NoClone> = Key { key: 7, _t: PhantomData };
    let j = k;
    let c = key_of(&k);
    println!("{} {} {}", k.key, j.key, c.key);
    let p = P(1, 'p');
    let arr = [p; 3];
    let f = move || p.0;
    println!("{} {} {} {}", p.0, arr[2].1, f(), f());
    let g = G(5u8, 6);
    let (a, b) = twice(&g);
    println!("{} {} {}", g.0, a.1, b.0);
    let r = R { s: "hi" };
    let r2 = r;
    let w = W(Q(2));
    let w2 = w;
    println!("{} {} {} {}", r.s, r2.s, w.0.0, w2.0.0);
}
