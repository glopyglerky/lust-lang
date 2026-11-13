// expect: L0303
// rustc E0502: a flatten's items borrow `w` through its front iterator, also when they are
// collected through `by_ref()` or come back from a generic callee's `I::Item`.
fn first<I: Iterator>(mut i: I) -> Option<I::Item> {
    i.next()
}
fn main() {
    let mut w = vec![vec![1u32], vec![2]];
    let mut it = w.iter().flatten();
    let r: Vec<&u32> = it.by_ref().take(1).collect();
    let f = first(&mut it);
    w.clear();
    println!("{:?} {:?}", r, f);
}
