// expect: L0203
// Probe d_double__box_live_drop (probes4, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
fn last<'x>(it: Box<dyn Iterator<Item = &'x u32> + 'x>) -> Option<&'x u32> { it.last() }
fn first<'x>(it: &mut (dyn Iterator<Item = &'x u32> + 'x)) -> Option<&'x u32> { it.next() }
fn mk(c: &C) -> Box<dyn DoubleEndedIterator<Item = &u32> + '_> { Box::new(c.v.iter()) }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    let it = mk(r);
    c.n = 1;
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
