// expect: L0203
// rustc E0658: an elided lifetime in an argument-position `impl Trait`
// (anonymous lifetimes in `impl Trait` are unstable).
fn f(a: impl IntoIterator<Item = &i64>) -> usize {
    a.into_iter().count()
}
fn main() {
    println!("{}", f(&[1i64]));
}
