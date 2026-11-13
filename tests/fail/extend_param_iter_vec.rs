// expect: L0306
// rustc E0597: `Extend<T>` for `Vec<T>` makes the element type equal to the type param
// iterator's `Item = &'a T`, which no call can shorten, so `&x0` must live for `'a`.
fn g<'a, T: Copy + 'a, I: Iterator<Item = &'a T>>(it: I, x0: T) -> usize {
    let mut v = Vec::new();
    v.extend(it);
    v.push(&x0);
    v.len()
}
fn main() {
    let s = vec![1u8, 2];
    println!("{}", g(s.iter(), 9u8));
}
