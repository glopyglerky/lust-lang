// `extend` with `&` items into a collection whose element type is still open:
// Extend<T> and Extend<&T> both fit until a later use fixes T (rustc keeps the
// choice open, so these all compile).
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

fn total(v: &Vec<u8>) -> u32 {
    v.iter().map(|&x| x as u32).sum()
}

fn main() {
    let (a, b) = (5u8, 7u8);
    // Vec: a later push of a value picks Extend<&T>.
    let mut v = Vec::new();
    v.extend([&a, &b]);
    v.push(1u8);
    println!("{:?}", v);
    // Vec: a later push of a reference picks Extend<T>.
    let mut r = Vec::new();
    r.extend([&a]);
    r.push(&b);
    println!("{:?}", r);
    // Vec: a later call fixes the element.
    let mut w = Vec::new();
    w.extend(vec![a, b].iter());
    println!("{}", total(&w));
    // An annotation after the call.
    let mut x = Vec::new();
    x.extend(std::iter::once(&b));
    let y: Vec<u8> = x;
    println!("{:?}", y);
    // VecDeque and BTreeSet: a later reference element.
    let mut d = VecDeque::new();
    d.extend([&a]);
    d.push_back(&b);
    println!("{:?}", d);
    let mut s = BTreeSet::new();
    s.extend([&b, &a]);
    s.insert(&a);
    println!("{:?}", s);
    // BTreeMap: a later reference pair.
    let mut m = BTreeMap::new();
    m.extend([(&a, &'x')]);
    m.insert(&b, &'y');
    println!("{:?}", m);
    // HashSet and HashMap: either form.
    let mut hs = HashSet::new();
    hs.extend([&a, &b]);
    hs.insert(9u8);
    let mut o: Vec<_> = hs.into_iter().collect();
    o.sort();
    println!("{:?}", o);
    let mut hr = HashSet::new();
    hr.extend([&a]);
    hr.insert(&b);
    println!("{}", hr.len());
    let mut hm = HashMap::new();
    hm.extend([(&a, &b)]);
    hm.insert(1u8, 2u8);
    let mut o: Vec<_> = hm.into_iter().collect();
    o.sort();
    println!("{:?}", o);
}
