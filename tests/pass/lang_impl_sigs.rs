// Impls of std (lang) traits in the spellings rustc accepts: `Self`,
// explicit lifetimes, `self: &Self`, trait arguments, `hash<H: Hasher>`
// and its `where` form, provided-method overrides, and an `Iterator`
// wrapper whose `Item` is the inner iterator's.
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::{Add, AddAssign, Neg};

#[derive(Debug, Clone, Copy)]
struct S(u8);
struct C(u32);
struct Words<'a>(&'a str);
#[derive(Debug)]
struct T(u8);
struct W<I>(I);

impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 3 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
}
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let s: &'a str = self.0;
        if s.is_empty() {
            return None;
        }
        self.0 = &s[1..];
        Some(&s[..1])
    }
}
impl<I: Iterator> Iterator for W<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        self.0.next()
    }
}
impl Add<u8> for S {
    type Output = S;
    fn add(self, o: u8) -> Self {
        S(self.0 + o)
    }
}
impl<'a> Add<&'a S> for &'a S {
    type Output = S;
    fn add(self, o: &'a S) -> S {
        S(self.0 + o.0)
    }
}
impl AddAssign for S {
    fn add_assign(&mut self, o: S) {
        self.0 += o.0;
    }
}
impl Neg for S {
    type Output = i16;
    fn neg(self) -> i16 {
        -(self.0 as i16)
    }
}
impl PartialEq<u8> for S {
    fn eq(&self, o: &u8) -> bool {
        self.0 == *o
    }
}
impl PartialEq for S {
    fn eq(&self, o: &Self) -> bool {
        self.0 == o.0
    }
    fn ne(&self, o: &S) -> bool {
        self.0 != o.0
    }
}
impl Eq for S {}
impl PartialOrd for S {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for S {
    fn cmp<'a>(&'a self, o: &'a Self) -> Ordering {
        self.0.cmp(&o.0)
    }
}
impl Hash for S {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.0);
    }
}
impl Hash for T {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u8(self.0);
    }
}
impl Default for T {
    fn default() -> Self {
        T(9)
    }
}
impl Clone for T {
    fn clone(self: &Self) -> T {
        T(self.0)
    }
}
impl Drop for C {
    fn drop(&mut self) {
        println!("drop C {}", self.0);
    }
}
struct Fnv(u64);
impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ *b as u64).wrapping_mul(0x100000001b3);
        }
    }
}
#[derive(Default, Clone)]
struct FnvB;
impl BuildHasher for FnvB {
    type Hasher = Fnv;
    fn build_hasher(&self) -> Fnv {
        Fnv(0xcbf29ce484222325)
    }
}

fn main() {
    let v: Vec<u32> = C(0).collect();
    let ws: Vec<&str> = Words("abc").collect();
    let mut w = W(vec![7u8, 8].into_iter());
    println!("{v:?} {ws:?} {:?}", w.next());
    let s = S(1) + 2;
    let mut t = &s + &S(4);
    t += S(1);
    println!("{:?} {:?} {} {} {} {:?}", s, t, -t, s == 3u8, s != t, s.cmp(&t));
    println!("{:?} {:?}", T::default(), T(4).clone());
    let mut m: HashMap<u8, u8, FnvB> = HashMap::default();
    m.insert(1, 2);
    m.insert(3, 4);
    println!("{:?} {}", m.get(&3), m.len());
}
