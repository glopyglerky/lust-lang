// core's `impl Iterator for &mut I`: adapters and consumers through
// `&mut it` and `by_ref()` step `it` in place; the loan ends after the call.
struct Cnt(u32);
impl Iterator for Cnt {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 6 { self.0 += 1; Some(self.0) } else { None }
    }
}
fn walk<I: Iterator<Item = u32>>(it: &mut I) -> u32 {
    let mut s = 0;
    for x in it { s += x; if s > 5 { break; } }
    s
}
fn total<I: Iterator<Item = u32>>(mut it: I) -> u32 { let a = walk(&mut it); let b = walk(&mut it); a * 100 + b }
fn exact<I: ExactSizeIterator>(i: I) -> usize { i.len() }
fn main() {
    let mut v = vec![1u32, 2, 3, 4, 5];
    let mut it = v.iter().copied();
    let first: Vec<u32> = (&mut it).take(2).collect();
    let n = it.by_ref().count();
    println!("{:?} {} {:?}", first, n, it.next());
    let mut it = v.iter();
    let xs: Vec<&u32> = it.by_ref().skip(1).step_by(2).collect();
    println!("{:?} {:?}", xs, it.next());
    let mut it = v.iter();
    println!("{} {:?}", exact(&mut it), it.next_back());
    let mut c = Cnt(0);
    println!("{} {:?}", walk(&mut c), c.next());
    println!("{} {}", total(Cnt(0)), total(vec![1u32, 2, 3, 4, 5, 6, 7].into_iter()));
    let mut r = 0u32..10;
    println!("{} {:?}", walk(&mut r), r);
    let s = String::from("héllo");
    let mut cs = s.chars();
    let a: String = (&mut cs).take(2).collect();
    let k = (&mut cs).count();
    println!("{} {} {:?}", a, k, cs.next());
    let mut it = v.iter();
    let mut got = Vec::new();
    for x in &mut it { got.push(*x); if *x == 2 { break; } }
    println!("{:?} {:?}", got, it.next());
    let mut it = v.iter_mut();
    let a = (&mut it).next();
    let b = (&mut it).next();
    if let (Some(a), Some(b)) = (a, b) { *a += *b; }
    println!("{:?}", v);
}
