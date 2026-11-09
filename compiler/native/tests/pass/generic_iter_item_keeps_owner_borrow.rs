// A generic callee whose result names `I::Item`, called with I = `&mut it` over a borrowing
// iterator: the items borrow the iterator's source, not the `&mut it` reborrow, so `it` is free
// again after the call while the result lives.
fn ext<I: IntoIterator>(i: I) -> Vec<I::Item> {
    let mut v = Vec::new();
    v.extend(i);
    v
}
fn first<I: Iterator>(mut i: I) -> Option<I::Item> {
    i.next()
}
fn main() {
    let w = vec![7u8, 8, 9];
    let mut it = w.iter();
    let f = first(&mut it);
    println!("{:?} {:?}", f, it.next());
    let rest = ext(&mut it);
    println!("{:?} {:?}", rest, it.next());

    let mut it = w.iter();
    let g = first(it.by_ref());
    let n = it.next();
    println!("{:?} {:?}", g, n);

    let mut m = vec![1u32, 2, 3];
    let mut im = m.iter_mut();
    let a = first(&mut im).unwrap();
    let b = im.next().unwrap();
    *a += 10;
    *b += 20;
    println!("{:?}", m);

    let r;
    {
        let mut it = w.iter();
        r = first(&mut it);
    }
    println!("{:?}", r);
}
