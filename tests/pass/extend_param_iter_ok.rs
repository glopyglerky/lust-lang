// Fixed type param iterator items: a sink whose elements live as long as `'a` takes
// them, a `map` closure picks its own output region, `collect` gives a value that a move
// may shorten, and a builtin iterator argument (covariant) may be shortened at the call.
use std::collections::BTreeSet;
fn keep<'a, I: Iterator<Item = &'a u8>>(it: I, v: &mut Vec<&'a u8>, r: &'a u8) {
    v.extend(it);
    v.push(r);
}
fn mapped<'a, I: Iterator<Item = &'a u8>>(it: I, x0: u8) -> usize {
    let mut v = Vec::new();
    v.extend(it.map(|x| x));
    v.push(&x0);
    v.iter().map(|x| **x as usize).sum()
}
fn moved<'a, I: Iterator<Item = &'a u8>>(it: I, x0: u8) -> usize {
    let v: BTreeSet<&u8> = it.collect();
    let mut w = v;
    w.insert(&x0);
    w.len()
}
fn slice<'a>(it: std::slice::Iter<'a, u8>, x0: u8) -> usize {
    let mut v = Vec::new();
    v.extend(it);
    v.push(&x0);
    v.len()
}
fn main() {
    let s = vec![1u8, 2, 3];
    let r = 7u8;
    let mut v = Vec::new();
    keep(s.iter(), &mut v, &r);
    println!("{:?} {} {} {}", v, mapped(s.iter(), 9), moved(s.iter(), 9), slice(s.iter(), 9));
}
