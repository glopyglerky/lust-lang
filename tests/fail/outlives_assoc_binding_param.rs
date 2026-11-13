// expect: L0617
// rustc E0309: `Item = &'a T` in a bound needs `T: 'a`; nothing implies it.
fn first<'a, T: Copy, I: IntoIterator<Item = &'a T>>(i: I) -> Option<T> {
    i.into_iter().next().copied()
}
fn main() {
    let v = vec![1, 2];
    println!("{:?}", first(&v));
}
