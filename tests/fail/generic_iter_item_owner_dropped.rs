// expect: L0311
// rustc E0505: the items of `first(&mut it)` borrow `w` (I = &mut slice::Iter<'w, u8>), so `w`
// cannot move while `f` lives.
fn first<I: Iterator>(mut i: I) -> Option<I::Item> {
    i.next()
}
fn main() {
    let w = vec![7u8, 8, 9];
    let mut it = w.iter();
    let f = first(&mut it);
    drop(w);
    println!("{:?}", f);
}
