// expect: L0203
// Probe d_gen__box_live_drop (probes4, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
fn last<'x>(it: Box<dyn Iterator<Item = &'x u32> + 'x>) -> Option<&'x u32> { it.last() }
fn first<'x>(it: &mut (dyn Iterator<Item = &'x u32> + 'x)) -> Option<&'x u32> { it.next() }
fn mk<T>(v: &[T]) -> Box<dyn Iterator<Item = &T> + '_> { Box::new(v.iter()) }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    let it = mk(&r.v);
    c.n = 1;
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
