// `for` over `impl<'a, K, V> IntoIterator for &'a M<K, V>` where the type
// args hold 'static regions: the loop's temps take check's types, not the
// impl's lifetimes (R10).
use std::rc::Rc;
trait FH { fn name(&self) -> String; }
struct A;
impl FH for A { fn name(&self) -> String { "a".into() } }
struct M<K, V> { e: Vec<(K, V)> }
struct It<'a, K, V> { i: std::slice::Iter<'a, (K, V)> }
impl<'a, K, V> Iterator for It<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.i.next().map(|(k, v)| (k, v)) }
}
impl<'a, K, V> IntoIterator for &'a M<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = It<'a, K, V>;
    fn into_iter(self) -> It<'a, K, V> { It { i: self.e.iter() } }
}
static K0: u32 = 4;
fn show(v: &[&'static str]) -> usize { v.len() }
fn need_u(v: &'static u32) -> usize { *v as usize }
fn collect(out: &mut Vec<String>, open: &M<u32, Rc<dyn FH>>) {
    for (_k, f) in open { out.push(f.name()); }
}
fn sum<'q>(m: &'q M<u32, &'static u32>) -> usize { let mut n = 0; for (_k, v) in m { n += need_u(*v); } n }
struct H { m: M<u32, &'static u32> }
impl H { fn run(&self) -> usize { let mut n = 0; for (_k, v) in &self.m { n += need_u(*v); } n } }
fn main() {
    let m: M<u32, Vec<&'static str>> = M { e: vec![(1, vec!["a"])] };
    let mut n = 0;
    for (_k, v) in &m { n += show(v); }
    let o: M<u32, Rc<dyn FH>> = M { e: vec![(1, Rc::new(A))] };
    let mut v = Vec::new();
    collect(&mut v, &o);
    let h = H { m: M { e: vec![(1, &K0)] } };
    println!("{n} {v:?} {} {}", sum(&h.m), h.run());
}
