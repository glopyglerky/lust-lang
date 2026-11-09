// Method calls autoderef through user Deref impls (rustc's autoderef):
// inherent and slice methods on the target, DerefMut for `&mut self`
// methods, a generic impl (SmallVec's `Deref<Target = [T]>` shape), a
// chain of impls, dyn and str targets, closures, and generic callers.
use std::ops::{Deref, DerefMut};
struct N { v: u32 }
impl N { fn get(&self) -> u32 { self.v } fn bump(&mut self) { self.v += 1; } }
struct W { n: N, tag: u8 }
impl Deref for W { type Target = N; fn deref(&self) -> &N { &self.n } }
impl DerefMut for W { fn deref_mut(&mut self) -> &mut N { &mut self.n } }
struct G<T> { items: Vec<T> }
impl<T> Deref for G<T> { type Target = [T]; fn deref(&self) -> &[T] { &self.items } }
impl<T> DerefMut for G<T> { fn deref_mut(&mut self) -> &mut [T] { &mut self.items } }
fn total(g: &G<u32>) -> u32 { g.iter().sum() }
fn part1() {
    let mut w = W { n: N { v: 4 }, tag: 1 };
    println!("{} {}", w.get(), w.tag);
    w.bump();
    println!("{}", w.get());
    let mut g = G { items: vec![3u32, 1, 2] };
    println!("{} {:?} {} {:?}", g.len(), g.first(), g.contains(&2), g.get(1));
    for x in g.iter_mut() { *x *= 10; }
    println!("{:?} {}", g.items, total(&g));
    let r = &g;
    println!("{} {}", r.len(), r.is_empty());
    for x in g.iter() { print!("{} ", x); }
    println!();
    let s = G { items: vec![String::from("a"), String::from("bc")] };
    println!("{:?}", s.iter().map(|x| x.len()).collect::<Vec<_>>());
}

trait Tr { fn hi(&self) -> String; }
struct A;
impl Tr for A { fn hi(&self) -> String { String::from("a") } }
struct Holder { b: Box<dyn Tr> }
impl Deref for Holder { type Target = dyn Tr; fn deref(&self) -> &(dyn Tr + 'static) { &*self.b } }
struct Name { s: String }
impl Deref for Name { type Target = str; fn deref(&self) -> &str { &self.s } }
fn count<T: Copy>(g: &G<T>) -> usize { g.len() + g.iter().count() }
fn part2() {
    let h = Holder { b: Box::new(A) };
    println!("{}", h.hi());
    let n = Name { s: String::from("hello") };
    println!("{} {} {}", n.len(), n.to_uppercase(), n.starts_with("he"));
    println!("{} {}", count(&G { items: vec![1u8, 2] }), count(&G { items: vec!['x'] }));
}

struct MM {
    v: u8,
}
impl MM {
    fn m(&self) -> u8 {
        self.v
    }
}
struct NN {
    m: MM,
}
impl Deref for NN {
    type Target = MM;
    fn deref(&self) -> &MM {
        &self.m
    }
}
struct WW {
    n: NN,
}
impl Deref for WW {
    type Target = NN;
    fn deref(&self) -> &NN {
        &self.n
    }
}
fn part3() {
    let w = WW { n: NN { m: MM { v: 9 } } };
    println!("{}", w.m());
}

fn main() {
    part1();
    part2();
    part3();
}
