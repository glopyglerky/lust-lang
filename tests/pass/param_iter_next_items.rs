// `next`, `nth`, `last`, `max`, `next_back` through a type param's own `Iterator<Item = X>`
// bound give `X`, which holds no borrow of the iterator: the items outlive it, and a
// callee taking `&mut I` returns items that outlive the reborrow.
fn pull<'a, I: Iterator<Item = &'a u32>>(mut it: I, out: &mut Vec<&'a u32>) {
    while let Some(x) = it.next() {
        out.push(x);
    }
}
fn ends<'a, I: DoubleEndedIterator<Item = &'a u32> + Clone>(mut it: I, out: &mut Vec<&'a u32>) {
    let a = it.next();
    let b = it.next_back();
    let m = it.clone().max();
    let n = it.nth(0);
    let l = it.last();
    out.extend(a.into_iter().chain(b).chain(m).chain(n).chain(l));
}
fn first<'a, I: Iterator<Item = &'a u32>>(it: &mut I) -> Option<&'a u32> {
    it.next()
}
fn main() {
    let n = [1u32, 2, 3, 4, 5];
    let mut v = Vec::new();
    pull(n.iter(), &mut v);
    ends(n.iter(), &mut v);
    let r;
    {
        let mut it = n.iter();
        r = first(&mut it);
    }
    println!("{:?} {:?}", v, r);
}
