// expect: L0612
// rustc E0599: a written `Cloned<slice::Iter<A>>` is no iterator without `A: Clone`, so
// `count` does not apply. lust asks the written type to be an iterator.
struct A(u8);
fn f(x: std::iter::Cloned<std::slice::Iter<'static, A>>) -> usize {
    x.count()
}
fn main() {
    println!("ok");
}
