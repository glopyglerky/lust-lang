// A builtin store relates a plain stored value's parts one by one: a Vec of
// pairs with two lifetimes takes a pair without tying the two lifetimes,
// and a generic push at `T = &mut &u8` builds (its instance used to tie the
// `&mut` reborrow to the element's lifetimes). Library containers of `&mut`
// build (their `Clone` impls do not apply), and fn pointers with a
// `'static` output extend a Vec of shorter outputs.
use std::collections::{BTreeMap, VecDeque};

fn add<'a, 'b>(v: &mut Vec<(&'a u8, &'b mut u8)>, p: &'a u8, q: &'b mut u8) {
    v.push((p, q));
    v.len();
}

fn put<'a, 'b>(v: &mut Vec<&'a mut &'b u8>, x: &'a mut &'b u8) {
    v.push(x);
    v.len();
}

fn push<T>(buf: &mut Vec<T>, value: T) {
    buf.push(value);
    buf.len();
}

fn first(x: &u8) -> &'static u8 {
    if *x > 0 { &1 } else { &0 }
}

fn fill<'b>(fs: &mut Vec<fn(&u8) -> &'b u8>, f: fn(&u8) -> &'static u8) {
    fs.extend([f]);
    fs.extend(std::iter::once(f));
}

fn main() {
    let a = 1u8;
    let mut b = 2u8;
    let mut v = Vec::new();
    add(&mut v, &a, &mut b);
    *v[0].1 += 10;
    println!("{} {}", v[0].0, v[0].1);

    let s = 3u8;
    let (mut r, mut r2): (&u8, &u8) = (&s, &s);
    let mut w = Vec::new();
    put(&mut w, &mut r);
    push(&mut w, &mut r2);
    println!("{}", w.len());

    let mut x = 4u8;
    let mut dq: VecDeque<&mut u8> = VecDeque::new();
    dq.push_back(&mut x);
    **dq.front_mut().unwrap() += 1;
    let mut y = 5u8;
    let mut m: BTreeMap<u8, &mut &u8> = BTreeMap::new();
    let mut ry: &u8 = &y;
    m.insert(1, &mut ry);
    println!("{} {}", dq.len(), m.len());
    drop(dq);
    y += 1;
    println!("{x} {y}");

    let mut fs = Vec::new();
    fill(&mut fs, first);
    println!("{} {}", fs.len(), fs[0](&a));
}
