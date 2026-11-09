// A `&mut I` over a builtin iterator is an iterator itself (core's `impl Iterator for &mut I`):
// `r.into_iter()`, `extend(&mut it)` on Vec, VecDeque and String, and the same through a generic
// `I: IntoIterator` instance (tf-borrow p53: check passed, build failed).
fn take2<I: IntoIterator>(i: I) -> Vec<I::Item> {
    i.into_iter().take(2).collect()
}
fn ext<I: IntoIterator>(i: I) -> Vec<I::Item> {
    let mut v = Vec::new();
    v.extend(i);
    v
}
fn main() {
    let mut a = [1u32, 2, 3, 4, 5].into_iter();
    let r = &mut a;
    let s: u32 = r.into_iter().take(1).sum();
    println!("{} {:?}", s, take2(&mut a));
    let mut v = vec![0u32];
    v.extend(&mut a);
    println!("{:?} {:?}", v, a.next());
    let w = vec![7u8, 8, 9];
    let mut it = w.iter();
    let mut d: std::collections::VecDeque<&u8> = std::collections::VecDeque::new();
    d.extend(it.by_ref().take(2));
    let rest = ext(&mut it);
    println!("{:?} {:?}", d, rest);
    println!("{:?}", it.next());
    let mut c = "abc".chars();
    let mut st = String::new();
    st.extend(&mut c);
    println!("{}", st);
}
