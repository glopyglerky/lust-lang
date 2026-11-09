// `I: IntoIterator` bounds as core declares the trait: the item bound
// `IntoIter: Iterator<Item = Self::Item>` makes `i.into_iter().next()`
// check in a template; Vec, &Vec, &mut Vec, &[T], and &mut [T] satisfy
// the bound (their literals take the bound's Item); and `into_iter()`
// on an iterator is core's blanket identity.
use std::convert::AsRef;

struct C {
    n: u32,
}
impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}

fn sum<I: IntoIterator<Item = u8>>(i: I) -> u8 {
    let mut it = i.into_iter();
    let mut s = 0u8;
    while let Some(x) = it.next() {
        s += x;
    }
    for x in it {
        s += x;
    }
    s
}

struct Bag {
    items: Vec<u32>,
}
impl Bag {
    fn add_all<I: IntoIterator<Item = u32>>(&mut self, xs: I) {
        for x in xs {
            self.items.push(x);
        }
    }
}

fn count_strs(xs: impl IntoIterator<Item = String>) -> usize {
    let mut n = 0;
    let mut it = xs.into_iter();
    while let Some(s) = it.next() {
        n += s.len();
    }
    n
}

fn firsts<I>(xs: I) -> Vec<char>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut out = Vec::new();
    for s in xs {
        if let Some(c) = s.as_ref().chars().next() {
            out.push(c);
        }
    }
    out
}

fn total<'a, I: IntoIterator<Item = &'a u8>>(i: I) -> u32 {
    let mut s = 0u32;
    for x in i {
        s += *x as u32;
    }
    s
}

fn bump<'a, I: IntoIterator<Item = &'a mut u8>>(i: I) {
    for x in i {
        *x += 1;
    }
}

fn drain<I: Iterator<Item = u32>>(i: I) -> u32 {
    let mut it = i.into_iter();
    let mut s = 0;
    while let Some(x) = it.next() {
        s += x;
    }
    s
}

fn main() {
    println!("{} {}", sum(vec![1, 2, 3]), sum(vec![4u8, 5]));
    let mut b = Bag { items: Vec::new() };
    b.add_all(vec![1, 2, 3]);
    b.add_all(C { n: 1 });
    println!("{:?}", b.items);
    println!("{} {:?}", count_strs(vec!["ab".to_string(), "cde".to_string()]), firsts(vec!["xy", "zw"]));
    let mut v = vec![1u8, 2, 3];
    let s = &v[1..];
    println!("{} {}", total(&v), total(s));
    bump(&mut v);
    bump(&mut v[..2]);
    println!("{:?}", v);
    let w = vec![7u8, 8];
    let mut c = w.iter().into_iter();
    let mut d = (0..3u8).into_iter();
    let mut e = C { n: 0 }.into_iter();
    println!("{:?} {:?} {:?} {}", c.next(), d.next(), e.next(), drain(C { n: 1 }));
}
