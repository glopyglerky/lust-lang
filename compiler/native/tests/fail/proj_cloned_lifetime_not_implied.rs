// expect: L0617
// rustc E0309: `<Cloned<slice::Iter<'a, T>> as Iterator>::Item` is `T`; the impl needs
// `T: 'a`, and the signature implies only what the normalized type needs.
fn f<'a, T: Clone>(x: <std::iter::Cloned<std::slice::Iter<'a, T>> as Iterator>::Item) -> T {
    x
}
fn main() {
    println!("{}", f(1u8));
}
